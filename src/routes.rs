use anyhow::Result;
use log::debug;
use rand::seq::IndexedRandom;
use rtfw_http::http::response_status_codes::HttpStatusCode;
use rtfw_http::http::{HttpRequest, HttpResponse, HttpResponseBuilder};
use rtfw_http::router::RoutingData;
use std::fs;

use crate::{http_helpers, utils};

pub fn get_index(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let user = http_helpers::get_logged_in_user(request)?;
    let login_section = if user.is_some() {
        format!(
            "<li><a href=\"/auth/me\"> Profile</a></li><li><a href=\"/auth/logout\">󰍃 Logout</a></li>"
        )
        .to_string()
    } else {
        "<li><a href=\"/auth/login\">󰍂 Login</a></li>".to_string()
    };

    let name = match user {
        Some(user) => user.username,
        None => "World".to_string(),
    };

    let greet_msg = format!("Hello {}!", name);

    let body = utils::load_view("index")?
        .replace("{{GREET_MSG}}", &greet_msg)
        .replace("{{LOGIN_DYNA_BLOCK}}", &login_section)
        .replace("{{AOT_CALENDAR}}", &generate_calendar_body());
    HttpResponseBuilder::new().set_html_body(&body).build()
}

fn generate_calendar_body() -> String {
    let mut body = String::from("<div></p>");
    let today = utils::get_current_day();
    for day in 1..=25 {
        let day_link = if day <= today {
            format!("<a class=\"day-link\" href=\"/day/{day}\">{day}</a>")
        } else {
            format!("<a class=\"day-link disabled\" href=\"/day/{day}\">{day}</a>")
        };

        body.push_str(&day_link);
        if day % 5 == 0 {
            body.push_str("</p><p>");
        }
    }

    body.push_str("</p></div>");
    body
}

pub fn get_about(_request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let body = utils::load_view("about")?;
    HttpResponseBuilder::new().set_html_body(&body).build()
}

pub fn catcher_get_404(
    _request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let body = utils::load_view("404")?;
    let catchphrases: Vec<_> = fs::read_to_string("src/assets/404_phrases.md")?
        .lines()
        .map(String::from)
        .collect();

    let phrase = catchphrases.choose(&mut rand::rng()).unwrap();
    let rendered_phrase = utils::markdown_to_html(phrase)?;
    let body = body.replace("{{CATCHPHRASE}}", &rendered_phrase);
    debug!(
        "someone got lost, giving them the catch all route and a catchphrase: {rendered_phrase}"
    );

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::NotFound)
        .set_html_body(&body)
        .build()
}
