use anyhow::{Context, Result};
use chrono::Local;
use log::debug;
use rtfw_http::{
    http::{
        HttpCookie, HttpRequest, HttpResponse, HttpResponseBuilder,
        response_status_codes::HttpStatusCode,
    },
    router::RoutingData,
};

use crate::{
    database::user_repository::UserRepository,
    http_helpers::{self, redirect},
    models::{
        discord_user_response::DiscordUserResponse, microsoft_user_response::MicrosoftUserResponse,
    },
    oauth2, security, utils,
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
    UserRepository::update_user(user)?;

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

    let mut guess_data_block = String::from("<ul>");
    let current_day = utils::get_current_day();
    let mut total_pts = 0;
    for day in 1..=current_day {
        let txt = match user.guess_data.get(&day) {
            Some(guess_data) => {
                let points = guess_data.points;
                total_pts += points;
                format!("<li>Day {day} => {points} ⭐</li>")
            }
            None => format!("<li>Day {day} => 0 ⚫</li>"),
        };
        guess_data_block.push_str(&txt);
    }
    guess_data_block.push_str("</ul>");
    guess_data_block.push_str(&format!("<p>Total: {total_pts} ⭐</p>"));

    let body = utils::load_view("profile")?
        .replace("{{USERNAME}}", &user.username)
        .replace("{{GUESS_DATA_BLOCK}}", &guess_data_block);
    HttpResponseBuilder::new().set_html_body(&body).build()
}

pub fn get_oauth2_login(
    request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let provider = request.query.get("idp").context("IDP should be provided")?;
    let oauth2_config = security::get_oauth2_provider_config(provider)?;

    oauth2::redirect_to_authorize(oauth2_config)
}

pub fn get_microsoft_oauth2_redirect(
    request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let oauth2_config = security::get_oauth2_provider_config("microsoft")?;
    let code = request
        .query
        .get("code")
        .context("should have a code")?
        .to_owned();

    let oauth2_response = oauth2::exchange_token(&code, oauth2_config)?;
    let user = MicrosoftUserResponse::create_app_user(&oauth2_response)?;

    match UserRepository::get_user_by_id(&user.id)? {
        Some(mut existing_user) => {
            debug!("existing user logged in: {existing_user:#?}");
            existing_user.access_token = user.access_token;
            existing_user.refresh_token = user.refresh_token;
            existing_user.access_token_expire_at = user.access_token_expire_at;
            UserRepository::update_user(existing_user)?;
        }
        None => {
            debug!("newly created user: {user:#?}");
            UserRepository::create_user(user)?;
        }
    }

    let bearer_cookie = HttpCookie::new(http_helpers::BEARER_COOKIE, &oauth2_response.access_token)
        .set_path(Some("/"))
        .set_max_age(Some(oauth2_response.expires_in as i32));

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::Found)
        .set_cookie(bearer_cookie)
        .set_header("Location", "/")
        .build()
}

pub fn get_discord_oauth2_redirect(
    request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let oauth2_config = security::get_oauth2_provider_config("discord")?;
    let code = request
        .query
        .get("code")
        .context("should have a code")?
        .to_owned();

    let oauth2_response = oauth2::exchange_token(&code, oauth2_config)?;
    let user = DiscordUserResponse::create_app_user(&oauth2_response)?;

    match UserRepository::get_user_by_id(&user.id)? {
        Some(mut existing_user) => {
            debug!("existing user logged in: {existing_user:#?}");
            existing_user.access_token = user.access_token;
            existing_user.refresh_token = user.refresh_token;
            existing_user.access_token_expire_at = user.access_token_expire_at;
            UserRepository::update_user(existing_user)?;
        }
        None => {
            debug!("newly created user: {user:#?}");
            UserRepository::create_user(user)?;
        }
    }

    let bearer_cookie = HttpCookie::new(http_helpers::BEARER_COOKIE, &oauth2_response.access_token)
        .set_path(Some("/"))
        .set_max_age(Some(oauth2_response.expires_in as i32));

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::Found)
        .set_cookie(bearer_cookie)
        .set_header("Location", "/")
        .build()
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
