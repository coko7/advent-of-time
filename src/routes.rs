use anyhow::Result;
use log::debug;
use rand::seq::IndexedRandom;
use rtfw_http::http::response_status_codes::HttpStatusCode;
use rtfw_http::http::{HttpRequest, HttpResponse, HttpResponseBuilder};
use rtfw_http::router::RoutingData;
use serde::Serialize;
use serde_json::json;
use std::fs;

use crate::utils::Day;
use crate::{http_helpers, utils};

pub fn get_index(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let user = http_helpers::get_logged_in_user(request)?;
    let authenticated = user.is_some();
    let name = match user {
        Some(user) => user.username,
        None => "World".to_string(),
    };

    let greet_msg = format!("Hello {}!", name);

    let data = json!({
        "authenticated": authenticated,
        "greetMsg": greet_msg,
        "days": get_calendar_entries(),
    });
    let rendered = utils::render_view("index", &data)?;
    HttpResponseBuilder::new().set_html_body(&rendered).build()
}

#[derive(Serialize)]
pub struct CalendarEntry {
    pub day: Day,
    pub released: bool,
}

fn get_calendar_entries() -> Vec<CalendarEntry> {
    let today = utils::get_current_day();
    (1..=25)
        .map(|day| CalendarEntry {
            day,
            released: day <= today,
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
