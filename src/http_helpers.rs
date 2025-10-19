use anyhow::{Result, ensure};
use log::trace;
use rtfw_http::http::{
    HttpRequest, HttpResponse, HttpResponseBuilder, response_status_codes::HttpStatusCode,
};

use crate::{database, models::user::User, security};

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

pub fn is_logged_in(request: &HttpRequest) -> Result<bool> {
    trace!("try to get cookie bearer");
    trace!("{request:#?}");
    let bearer = match request.cookies.get(BEARER_COOKIE) {
        Some(val) => val,
        None => return Ok(false),
    };

    trace!("cookie bearer: {bearer:?}");
    let user = match database::get_user_by_bearer(&bearer.value)? {
        Some(user) => user,
        None => return Ok(false),
    };

    trace!("logged in user: {user:?}");
    security::is_user_token_valid(&user)
}

pub fn get_logged_in_user(request: &HttpRequest) -> Result<Option<User>> {
    let bearer = match request.cookies.get(BEARER_COOKIE) {
        Some(val) => val,
        None => return Ok(None),
    };

    let user = match database::get_user_by_bearer(&bearer.value)? {
        Some(user) => user,
        None => return Ok(None),
    };

    ensure!(security::is_user_token_valid(&user)?);
    Ok(Some(user))
}
