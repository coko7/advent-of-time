use anyhow::{Context, Result};
use log::{debug, warn};
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder, response_status_codes::HttpStatusCode},
    router::RoutingData,
};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    config::OAuth2Config,
    database::user_repository::UserRepository,
    http_helpers::{self, redirect},
    models::{
        discord_user_response::DiscordUserInfoHandler, github_user_response::GitHubUserInfoHandler,
        microsoft_user_response::MicrosoftUserInfoHandler,
        oauth_user_info_handler::OAuthUserInfoHandler,
    },
    oauth2, routes, security,
    utils::{self, capitalize},
};

pub fn get_login(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    if http_helpers::is_logged_in(request)? {
        return redirect("/auth/me");
    }

    let data = json!({
        "i18n": I18n::from_request(request).unwrap(),
    });

    let rendered = utils::render_view("login", &data)?;
    HttpResponseBuilder::new().set_html_body(&rendered).build()
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

pub fn get_oauth2_login(request: &HttpRequest, routing_data: &RoutingData) -> Result<HttpResponse> {
    let provider = request.query.get("idp").context("IDP should be provided")?;
    let oauth2_config = security::get_oauth2_provider_config(provider)?;

    if oauth2_config.enabled {
        oauth2::redirect_to_authorize(&oauth2_config)
    } else {
        routes::catcher_get_404(request, routing_data)
    }
}

fn prettify_error(error: &str) -> String {
    capitalize(error).replace("_", " ")
}

fn handle_access_token_response_error(request: &HttpRequest) -> Result<HttpResponse> {
    let error = request.query.get("error").context("error should be set")?;
    let error_description = request
        .query
        .get("error_description")
        .context("error_description should be set")?;

    warn!("oauth2 authorization response failed: {error} - {error_description}");
    let data = json!({
        "error": prettify_error(error),
        "error_description": error_description.replace("+", " "),
    });

    let rendered = utils::render_view("oauth2_error", &data)?;
    HttpResponseBuilder::new().set_html_body(&rendered).build()
}

fn oauth2_redirect<T: for<'a> Deserialize<'a>>(
    request: &HttpRequest,
    config: &OAuth2Config,
    response_creator: impl OAuthUserInfoHandler<T>,
) -> Result<HttpResponse> {
    if request.query.contains_key("error") {
        return handle_access_token_response_error(request);
    }

    let code = request
        .query
        .get("code")
        .context("should have a code")?
        .to_owned();

    let oauth2_response = oauth2::exchange_token(&code, config)?;
    let user = response_creator.create_app_user(&oauth2_response)?;

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

pub fn get_github_oauth2_redirect(
    request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let config = security::get_oauth2_provider_config("github")?;
    oauth2_redirect(request, &config, GitHubUserInfoHandler {})
}

pub fn get_microsoft_oauth2_redirect(
    request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let config = security::get_oauth2_provider_config("microsoft")?;
    oauth2_redirect(request, &config, MicrosoftUserInfoHandler {})
}

pub fn get_discord_oauth2_redirect(
    request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let config = security::get_oauth2_provider_config("discord")?;
    oauth2_redirect(request, &config, DiscordUserInfoHandler {})
}

#[derive(Serialize)]
struct I18n {
    title: String,
    login_with: String,
}

impl I18n {
    fn from_request(request: &HttpRequest) -> Result<I18n> {
        let user_locale = http_helpers::get_user_locale(request)?.to_str();
        Ok(I18n {
            title: t!("auth.title", locale = user_locale).to_string(),
            login_with: t!("auth.login_with", locale = user_locale).to_string(),
        })
    }
}
