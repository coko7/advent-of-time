use anyhow::Result;
use log::debug;
use rtfw_http::http::{HttpResponse, HttpResponseBuilder, response_status_codes::HttpStatusCode};
use std::collections::HashMap;

use crate::{config::OAuth2Config, models::oauth2_response::OAuth2Response};

pub fn redirect_to_authorize(config: OAuth2Config) -> Result<HttpResponse> {
    let authorize_url = config.authorize_url;
    let client_id = config.client_id;
    let redirect_uri = config.redirect_uri;
    let scope = config.scope;
    let encoded_redirect_uri: String =
        url::form_urlencoded::byte_serialize(redirect_uri.as_bytes()).collect();

    let authorize_request = format!(
        "{authorize_url}?client_id={client_id}&response_type=code&redirect_uri={encoded_redirect_uri}&scope={scope}"
    );

    debug!("{authorize_request}");
    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::Found)
        .set_header("Location", &authorize_request)
        .build()
}

pub fn exchange_token(code: &str, config: OAuth2Config) -> Result<OAuth2Response> {
    let token_url = config.token_url;
    let client_id = config.client_id;
    let redirect_uri = config.redirect_uri;
    let secret = config.secret;

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", secret);
    params.insert("grant_type", "authorization_code".to_owned());
    params.insert("code", code.to_string());
    params.insert("redirect_uri", redirect_uri);

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(token_url)
        .form(&params) // sends application/x-www-form-urlencoded data
        .send()?;

    debug!("exchange token response status: {}", response.status());
    let body = response.text()?;
    let oauth2_response = serde_json::from_str::<OAuth2Response>(&body)?;
    Ok(oauth2_response)
}

pub fn refresh_token(refresh_token: &str, config: OAuth2Config) -> Result<OAuth2Response> {
    let token_url = config.token_url;
    let client_id = config.client_id;
    let redirect_uri = config.redirect_uri;
    let secret = config.secret;

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", secret);
    params.insert("grant_type", "refresh_token".to_owned());
    params.insert("refresh_token", refresh_token.to_string());
    params.insert("redirect_uri", redirect_uri);

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(token_url)
        .form(&params) // sends application/x-www-form-urlencoded data
        .send()?;

    debug!("refresh token response status: {}", response.status());
    let body = response.text()?;
    let oauth2_response = serde_json::from_str::<OAuth2Response>(&body)?;
    Ok(oauth2_response)
}
