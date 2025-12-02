use anyhow::{Context, Result};
use log::debug;
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder, response_status_codes::HttpStatusCode},
    router::RoutingData,
};
use rust_i18n::t;
use serde::Serialize;
use serde_json::json;
use std::fs;

use crate::{
    database::picture_meta_repository::PictureMetaRepository,
    http_helpers, routes,
    utils::{self, Day},
};

pub fn get_single_day(request: &HttpRequest, routing_data: &RoutingData) -> Result<HttpResponse> {
    let day: Result<Option<u32>> = routing_data.get_value("id");
    if day.is_err() {
        debug!("invalid day ID format: {day:?}");
        return routes::catcher_get_404(request, routing_data);
    }

    let day = day.unwrap();
    if day.is_none() {
        debug!("no day ID provided");
        return routes::catcher_get_404(request, routing_data);
    }

    let day = day.unwrap();
    if !utils::is_day_valid(day) {
        debug!("invalid day requested: {day}");
        return routes::catcher_get_404(request, routing_data);
    }

    let body = load_day_view(request, day)?;
    HttpResponseBuilder::new().set_html_body(&body).build()
}

pub fn get_day_picture(_request: &HttpRequest, routing_data: &RoutingData) -> Result<HttpResponse> {
    let day: Result<Option<u32>> = routing_data.get_value("id");
    if day.is_err() {
        debug!("invalid day ID format for img: {day:?}");
        return HttpResponseBuilder::new()
            .set_status(HttpStatusCode::NotFound)
            .build();
    }

    let day = day.unwrap();
    if day.is_none() {
        debug!("no day ID provided for img");
        return HttpResponseBuilder::new()
            .set_status(HttpStatusCode::NotFound)
            .build();
    }

    let day = day.unwrap();
    if !utils::is_day_valid(day) {
        debug!("invalid day requested for img: {day}");
        return HttpResponseBuilder::new()
            .set_status(HttpStatusCode::NotFound)
            .build();
    }

    let picture = PictureMetaRepository::get_picture(day)?.context("should exist")?;
    let picture_path = picture.get_full_path();
    // let time_component = utils::extract_time_from_image(&img.path);
    // debug!("img meta: {time_component:#?}");
    let mime_type = mime_guess::from_path(&picture_path).first_or_octet_stream();

    if let Ok(bin_content) = fs::read(&picture_path) {
        debug!("valid day img returned: {:?}", picture_path);
        HttpResponseBuilder::new()
            .set_raw_body(bin_content)
            .set_content_type(mime_type.as_ref())
            .build()
    } else {
        debug!("invalid day img requested: {:?}", picture_path);
        HttpResponseBuilder::new()
            .set_status(HttpStatusCode::NotFound)
            .build()
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DayDto {
    pub id: Day,
    pub img_src: String,
    pub img_alt: String,
    pub date_hint: String,
    pub location_hint: Option<String>,
    pub guess_data: Option<GuessDataDto>,
    pub real_time: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GuessDataDto {
    pub time: String,
    pub points: u32,
}

fn load_day_view(request: &HttpRequest, day: u32) -> Result<String> {
    let day_img_src = format!("/day-pic/{day}");
    let picture_meta =
        PictureMetaRepository::get_picture(day)?.context("picture should exist bruh")?;

    let user = http_helpers::get_logged_in_user(request)?;
    let authenticated = user.is_some();
    let guess_data = match user {
        Some(user) if user.has_guessed(day) => {
            let guess_data = user.guess_data.get(&day).unwrap();
            Some(GuessDataDto {
                time: guess_data.time(),
                points: user.get_points(day)?,
            })
        }
        _ => None,
    };

    let solution_time = if guess_data.is_some() {
        Some(picture_meta.time_taken.clone())
    } else {
        None
    };

    let data = json!({
        "title": &format!("Day {day}"),
        "authenticated": authenticated,
        "day": DayDto {
            id: day,
            img_src: day_img_src,
            img_alt: format!("Image for day {day}"),
            date_hint: picture_meta.original_date,
            location_hint: picture_meta.location,
            guess_data,
            real_time: solution_time,
        },
        "i18n": I18n::from_request(request).unwrap(),
    });

    let rendered = utils::render_view("day", &data)?;
    Ok(rendered)
}

#[derive(Serialize)]
struct I18n {
    already_guessed: String,
    guess_today: String,
    hint_original_date: String,
    hint_location: String,
    hint_real_time: String,
    hint_your_guess: String,
    hint_your_points: String,
    check_progress: String,
    check_point_system: String,
    submit_text: String,
    login_required: String,
}

impl I18n {
    fn from_request(request: &HttpRequest) -> Result<I18n> {
        let user_locale = http_helpers::get_user_locale(request)?.to_str();
        Ok(I18n {
            already_guessed: t!("day.already_guessed", locale = user_locale).to_string(),
            guess_today: t!("day.guess_today", locale = user_locale).to_string(),
            hint_original_date: t!("day.hint_original_date", locale = user_locale).to_string(),
            hint_location: t!("day.hint_location", locale = user_locale).to_string(),
            hint_real_time: t!("day.hint_real_time", locale = user_locale).to_string(),
            hint_your_guess: t!("day.hint_your_guess", locale = user_locale).to_string(),
            hint_your_points: t!("day.hint_your_points", locale = user_locale).to_string(),
            check_progress: t!("day.check_progress", locale = user_locale).to_string(),
            check_point_system: t!("day.check_point_system", locale = user_locale).to_string(),
            submit_text: t!("day.submit_text", locale = user_locale).to_string(),
            login_required: t!("day.login_required", locale = user_locale).to_string(),
        })
    }
}
