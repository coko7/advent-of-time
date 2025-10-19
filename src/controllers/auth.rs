use anyhow::{Context, Result};
use chrono::Local;
use log::debug;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use rtfw_http::{
    http::{
        HttpCookie, HttpRequest, HttpResponse, HttpResponseBuilder,
        response_status_codes::HttpStatusCode,
    },
    router::RoutingData,
};
use std::{collections::HashMap, time::Duration};

use crate::{
    config::Config,
    database,
    http_helpers::{self, redirect},
    models::{
        discord_user_response::DiscordUserResponse, oauth2_response::OAuth2Response, user::User,
    },
    utils,
};

pub fn get_login(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    if http_helpers::is_logged_in(request)? {
        return redirect("/auth/me");
    }

    let body = utils::load_view("login")?;
    HttpResponseBuilder::new().set_html_body(&body).build()
}

pub fn get_logout(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let mut user = match http_helpers::get_logged_in_user(request)? {
        Some(user) => user,
        None => return redirect("/auth/login"),
    };

    user.access_token_expire_at = Local::now().into();
    user.access_token = String::new();
    user.refresh_token = String::new();
    database::update_user(user)?;

    let clear_bearer_cookie = HttpCookie::new(http_helpers::BEARER_COOKIE, "")
        .set_path(Some("/"))
        .set_max_age(Some(0));

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::Found)
        .set_cookie(clear_bearer_cookie)
        .set_header("Location", "/")
        .build()
}

pub fn get_me(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let user = match http_helpers::get_logged_in_user(request)? {
        Some(user) => user,
        None => return redirect("/auth/login"),
    };

    let json = serde_json::to_string(&user)?;
    HttpResponseBuilder::new().set_json_body(&json)?.build()
}

pub fn get_oauth2_login(
    _request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let config = Config::load_from_file()?;
    let discord_oauth2 = config.oauth2.discord;

    let authorize_url = discord_oauth2.authorize_url;
    let client_id = discord_oauth2.client_id;
    let redirect_uri = discord_oauth2.redirect_uri;
    let encoded_redirect_uri: String =
        url::form_urlencoded::byte_serialize(redirect_uri.as_bytes()).collect();

    let request = format!(
        "{authorize_url}?client_id={client_id}&response_type=code&redirect_uri={encoded_redirect_uri}&scope=identify"
    );

    println!("{request}");
    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::Found)
        .set_header("Location", &request)
        .build()
}

pub fn get_oauth2_redirect(
    request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let config = Config::load_from_file()?;
    let discord_oauth2 = config.oauth2.discord;
    let token_url = discord_oauth2.token_url;
    let client_id = discord_oauth2.client_id;
    let redirect_uri = discord_oauth2.redirect_uri;

    let code = request
        .query
        .get("code")
        .context("should have a code")?
        .to_owned();
    let secret = discord_oauth2.secret;

    let client = reqwest::blocking::Client::new();
    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", secret);
    params.insert("grant_type", "authorization_code".to_owned());
    params.insert("code", code);
    params.insert("redirect_uri", redirect_uri);

    let response = client
        .post(token_url)
        .form(&params) // sends application/x-www-form-urlencoded data
        .send()?;

    println!("Status: {}", response.status());
    let body = response.text()?;
    let oauth2_res = serde_json::from_str::<OAuth2Response>(&body)?;
    let user = create_user_from_discord(&oauth2_res)?;

    match database::get_user_by_id(&user.id)? {
        Some(mut existing_user) => {
            debug!("existing user logged in: {existing_user:#?}");
            existing_user.access_token = user.access_token;
            existing_user.refresh_token = user.refresh_token;
            existing_user.access_token_expire_at = user.access_token_expire_at;
            database::update_user(existing_user)?;
        }
        None => {
            debug!("newly created user: {user:#?}");
            database::create_user(user)?;
        }
    }

    let bearer_cookie = HttpCookie::new(http_helpers::BEARER_COOKIE, &oauth2_res.access_token)
        .set_path(Some("/"))
        .set_max_age(Some(oauth2_res.expires_in as i32));

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::Found)
        .set_cookie(bearer_cookie)
        .set_header("Location", "/")
        .build()
}

fn fetch_discord_user_info(access_token: &str) -> Result<DiscordUserResponse> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://discord.com/api/v10/users/@me")
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()?;

    let res = response.text()?;
    let res = serde_json::from_str::<DiscordUserResponse>(&res)?;
    Ok(res)
}

fn create_user_from_discord(oauth2_response: &OAuth2Response) -> Result<User> {
    let user_info = fetch_discord_user_info(&oauth2_response.access_token)?;
    debug!("{user_info:#?}");

    let now = Local::now();
    let expires_in = oauth2_response.expires_in - 30; // invalidate 30 seconds early
    let at_expires_at = now + Duration::from_secs(expires_in);

    Ok(User {
        id: user_info.id,
        username: user_info.username,
        guess_data: HashMap::new(),
        access_token: oauth2_response.access_token.to_owned(),
        access_token_expire_at: at_expires_at.into(),
        refresh_token: oauth2_response.refresh_token.to_owned(),
    })
}

// pub fn post_login(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
//     // Try to get real IP from reverse proxy custom header
//     let real_ip = if let Some(real_ip_header) = request.headers.get("X-Real-IP") {
//         real_ip_header.value.to_owned()
//     } else {
//         // else fallback to the IP of the underlying TCP socket
//         request.peer_ip.to_string()
//     };
//
//     debug!("login attempt from: {real_ip}");
//
//     let mut clients = utils::get_known_clients()?;
//     let client = clients.iter_mut().find(|client| client.ip_addr == real_ip);
//
//     let config = Config::load_from_file()?;
//     let now = Utc::now();
//     if let Some(client) = client {
//         let duration = now.signed_duration_since(client.last_request_at);
//         if duration.num_milliseconds().abs() < config.auth.login_cooldown.into() {
//             client.last_request_at = now;
//             utils::save_known_clients(clients)?;
//
//             return HttpResponseBuilder::new()
//                 .set_status(HttpStatusCode::TooManyRequests)
//                 .set_html_body("yo, slow down")
//                 .build();
//         }
//     } else {
//         let client = Client {
//             ip_addr: real_ip.clone(),
//             last_request_at: now,
//             bearer_token: None,
//         };
//         clients.push(client);
//         utils::save_known_clients(clients)?;
//     }
//
//     let mut clients = utils::get_known_clients()?;
//     let client = clients
//         .iter_mut()
//         .find(|client| client.ip_addr == real_ip)
//         .context("client should exist")?;
//
//     let body = request.get_str_body()?;
//     let kvp = body.split_once("=");
//
//     if kvp.is_none() {
//         client.last_request_at = now;
//         utils::save_known_clients(clients)?;
//         return HttpResponseBuilder::new()
//             .set_status(HttpStatusCode::BadRequest)
//             .build();
//     }
//
//     let kvp = kvp.unwrap();
//
//     // Force uppercase, its ok to compromise security for user experience
//     let secret = config.auth.secret.to_uppercase();
//
//     if kvp.0 != "code" {
//         client.last_request_at = now;
//         utils::save_known_clients(clients)?;
//         return HttpResponseBuilder::new()
//             .set_status(HttpStatusCode::BadRequest)
//             .build();
//     }
//
//     let case_i_input = kvp.1.to_uppercase();
//     if case_i_input != secret {
//         client.last_request_at = now;
//         utils::save_known_clients(clients)?;
//         return HttpResponseBuilder::new()
//             .set_status(HttpStatusCode::Unauthorized)
//             .build();
//     }
//
//     let auth_cookie_val = utils::generate_rand_str(32);
//     let cookie = HttpCookie::new(AUTH_COOKIE_NAME, &auth_cookie_val);
//
//     client.last_request_at = now;
//     client.bearer_token = Some(auth_cookie_val);
//     utils::save_known_clients(clients)?;
//
//     HttpResponseBuilder::new()
//         .set_cookie(cookie)
//         .set_status(HttpStatusCode::Found)
//         .set_header("Location", "/photos")
//         .build()
// }
