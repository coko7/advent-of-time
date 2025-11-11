use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use log::trace;
use rtfw_http::http::{
    HttpCookie, HttpRequest, HttpResponse, HttpResponseBuilder,
    response_status_codes::HttpStatusCode,
};

use crate::{
    database::user_repository::UserRepository,
    models::{oauth2_response::OAuth2Response, user::User},
    oauth2, security,
};

pub const BEARER_COOKIE: &str = "aot-bearer";

pub fn redirect(location: &str) -> Result<HttpResponse> {
    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::Found)
        .set_header("Location", location)
        .build()
}

pub fn bad_request() -> Result<HttpResponse> {
    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::BadRequest)
        .build()
}

pub fn bad_request_msg(message: &str) -> Result<HttpResponse> {
    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::BadRequest)
        .set_html_body(message)
        .build()
}

pub fn create_bearer_cookie(oauth2_response: &OAuth2Response) -> HttpCookie {
    HttpCookie::new(BEARER_COOKIE, &oauth2_response.access_token)
        .set_path(Some("/"))
        .set_http_only(true)
        .set_max_age(oauth2_response.expires_in.and_then(|num| {
            if num <= i32::MAX as u64 {
                Some(num as i32)
            } else {
                None
            }
        }))
}

pub fn create_clear_bearer_cookie() -> HttpCookie {
    let expired_date: Option<DateTime<Utc>> = Utc.timestamp_opt(0, 0).single();
    HttpCookie::new(BEARER_COOKIE, "")
        .set_path(Some("/"))
        .set_http_only(true)
        .set_expires(expired_date)
}

pub fn is_logged_in(request: &HttpRequest) -> Result<bool> {
    trace!("try to get cookie bearer");
    trace!("{request:#?}");
    let bearer = match request.cookies.get(BEARER_COOKIE) {
        Some(val) => val,
        None => return Ok(false),
    };

    trace!("cookie bearer: {bearer:?}");
    let user = match UserRepository::get_user_by_bearer(&bearer.value)? {
        Some(user) => user,
        None => return Ok(false),
    };

    trace!("logged in user: {user:?}");
    Ok(!security::has_access_token_expired(&user)?)
}

pub fn get_logged_in_user(request: &HttpRequest) -> Result<Option<User>> {
    let bearer = match request.cookies.get(BEARER_COOKIE) {
        Some(val) => val,
        None => return Ok(None),
    };

    let mut user = match UserRepository::get_user_by_bearer(&bearer.value)? {
        Some(user) => user,
        None => return Ok(None),
    };

    // WARNING: Probably not how things should be done but it's okay for this app
    // TODO: Handle the refresh flow in a more secure manner

    // if no refresh token exists, then we consider access token never expires
    if user.access_token_expire_at.is_none() || !security::has_access_token_expired(&user)? {
        return Ok(Some(user));
    }

    let refresh_token = user.refresh_token.to_owned().unwrap();
    let oauth2_config = security::get_oauth2_provider_config(&user.oauth_provider)?;
    let oauth2_res = oauth2::refresh_token(&refresh_token, oauth2_config)?;
    user.set_auth(&oauth2_res)?;

    Ok(Some(user))
}
