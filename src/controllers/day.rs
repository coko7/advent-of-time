use anyhow::{Context, Result};
use log::{debug, error};
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder, response_status_codes::HttpStatusCode},
    router::RoutingData,
};
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

    error!("TODO: make sure to remove EXIF from those images");
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
    pub original_date: String,
    pub guess_data: Option<GuessDataDto>,
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
                points: guess_data.points,
            })
        }
        _ => None,
    };

    let data = json!({
        "title": &format!("Day {day}"),
        "authenticated": authenticated,
        "day": DayDto {
            id: day,
            img_src: day_img_src,
            img_alt: format!("Image for day {day}"),
            original_date: picture_meta.original_date,
            guess_data,
        }
    });

    let rendered = utils::render_view("day", &data)?;
    Ok(rendered)
}
