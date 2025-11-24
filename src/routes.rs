use anyhow::Result;
use chrono::Utc;
use log::debug;
use rand::seq::IndexedRandom;
use rtfw_http::http::response_status_codes::HttpStatusCode;
use rtfw_http::http::{HttpRequest, HttpResponse, HttpResponseBuilder};
use rtfw_http::router::RoutingData;
use rust_i18n::t;
use serde::Serialize;
use serde_json::json;
use std::fs;

use crate::models::user::User;
use crate::utils::{Day, load_i18n_for_user};
use crate::{http_helpers, utils};

pub fn get_index(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let user = http_helpers::get_logged_in_user(request)?;
    let authenticated = user.is_some();
    let name = match &user {
        Some(user) => user.username.to_owned(),
        None => "World".to_string(),
    };

    let greet_msg = format!("Hello {}!", name);

    let data = json!({
        "authenticated": authenticated,
        "greetMsg": greet_msg,
        "days": get_calendar_entries(user.as_ref()),
        "i18n": get_i18n(request),
    });
    let rendered = utils::render_view("index", &data)?;
    HttpResponseBuilder::new().set_html_body(&rendered).build()
}

fn get_i18n(request: &HttpRequest) -> I18n {
    I18n {
        title: load_i18n_for_user("title", request).unwrap(),
        edition: load_i18n_for_user("edition", request).unwrap(),
        about: load_i18n_for_user("index.about", request).unwrap(),
        profile: load_i18n_for_user("index.profile", request).unwrap(),
        logout: load_i18n_for_user("index.logout", request).unwrap(),
        login: load_i18n_for_user("index.login", request).unwrap(),
        leaderboard: load_i18n_for_user("index.leaderboard", request).unwrap(),
    }
}

#[derive(Serialize)]
struct I18n {
    title: String,
    edition: String,
    about: String,
    profile: String,
    logout: String,
    login: String,
    leaderboard: String,
}

#[derive(Serialize)]
pub struct CalendarEntry {
    pub day: Day,
    pub released: bool,
    pub guessed: bool,
}

fn get_calendar_entries(user: Option<&User>) -> Vec<CalendarEntry> {
    let utc_now = Utc::now();
    (1..=25)
        .map(|day| CalendarEntry {
            day,
            guessed: if let Some(user) = user {
                user.has_guessed(day)
            } else {
                false
            },
            released: utils::is_picture_released(utc_now, day),
        })
        .collect()
}

pub fn get_about(_request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let body = utils::load_view("about")?;
    HttpResponseBuilder::new().set_html_body(&body).build()
}

pub fn catcher_get_404(
    _request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let catchphrases: Vec<_> = fs::read_to_string("src/assets/404_phrases.md")?
        .lines()
        .map(String::from)
        .collect();

    let phrase = catchphrases.choose(&mut rand::rng()).unwrap();
    let rendered_phrase = utils::markdown_to_html(phrase)?;
    debug!(
        "someone got lost, giving them the catch all route and a catchphrase: {rendered_phrase}"
    );
    let rendered = utils::render_view("404", &json!({"catchphrase": rendered_phrase}))?;

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::NotFound)
        .set_html_body(&rendered)
        .build()
}
