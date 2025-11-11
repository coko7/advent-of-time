use anyhow::{Context, Result};
use log::debug;
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder, response_status_codes::HttpStatusCode},
    router::RoutingData,
};
use serde::Serialize;
use serde_json::json;

use crate::{
    database::user_repository::UserRepository,
    http_helpers::{self, redirect},
    models::{
        discord_user_response::DiscordUserResponse, github_user_response::GitHubUserResponse,
        microsoft_user_response::MicrosoftUserResponse, user::User,
    },
    oauth2, security,
    utils::{self, Day},
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

    user.clear_auth()?;
    UserRepository::update_user(user)?;

    let clear_bearer_cookie = http_helpers::create_clear_bearer_cookie();
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

    let data = json!({
        "username": &user.username,
        "days": get_user_guess_days(&user),
        "total_score": user.get_total_score(),
    });
    let rendered = utils::render_view("profile", &data)?;
    HttpResponseBuilder::new().set_html_body(&rendered).build()
}

#[derive(Debug, Serialize)]
struct UserGuessDay {
    pub day: Day,
    pub guessed: bool,
    pub time: String,
    pub points: u32,
}

fn get_user_guess_days(user: &User) -> Vec<UserGuessDay> {
    let current_day = utils::get_current_day();
    (1..=current_day)
        .map(|d| match user.guess_data.get(&d) {
            Some(guess) => UserGuessDay {
                day: d,
                guessed: true,
                time: guess.time(),
                points: guess.points,
            },
            None => UserGuessDay {
                day: d,
                guessed: false,
                time: String::new(),
                points: 0,
            },
        })
        .collect::<Vec<_>>()
}

pub fn get_oauth2_login(
    request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let provider = request.query.get("idp").context("IDP should be provided")?;
    let oauth2_config = security::get_oauth2_provider_config(provider)?;

    oauth2::redirect_to_authorize(oauth2_config)
}

pub fn get_github_oauth2_redirect(
    request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let oauth2_config = security::get_oauth2_provider_config("github")?;
    let code = request
        .query
        .get("code")
        .context("should have a code")?
        .to_owned();

    let oauth2_response = oauth2::exchange_token(&code, oauth2_config)?;
    let user = GitHubUserResponse::create_app_user(&oauth2_response)?;

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

    let bearer_cookie = http_helpers::create_bearer_cookie(&oauth2_response);

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::Found)
        .set_cookie(bearer_cookie)
        .set_header("Location", "/")
        .build()
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

    let bearer_cookie = http_helpers::create_bearer_cookie(&oauth2_response);

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

    let bearer_cookie = http_helpers::create_bearer_cookie(&oauth2_response);

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::Found)
        .set_cookie(bearer_cookie)
        .set_header("Location", "/")
        .build()
}
