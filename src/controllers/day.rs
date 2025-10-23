use anyhow::{Context, Result};
use log::{debug, error};
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder, response_status_codes::HttpStatusCode},
    router::RoutingData,
};
use std::fs;

use crate::{
    database::picture_meta_repository::PictureMetaRepository, http_helpers, routes, utils,
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

fn load_day_view(request: &HttpRequest, day: u32) -> Result<String> {
    let day_img_src = format!("/day-pic/{day}");
    let picture_meta =
        PictureMetaRepository::get_picture(day)?.context("picture should exist bruh")?;

    let user = http_helpers::get_logged_in_user(request)?;
    let dynamic_form = match user {
        Some(user) => {
            if user.has_guessed(day) {
                let guess_data = user.guess_data.get(&day).unwrap();
                format!(
                    r#"
                        <p>You guessed: {}:{}</p>
                        <p>Your points for this guess: {}</p>
                    "#,
                    guess_data.hm.0, guess_data.hm.1, guess_data.points
                )
            } else {
                r#"
                <form id="guess-daily-picture">
                <p>
                    <label for="time-guess">Your guess:</label>
                    <input id="time-guess" name="time-guess" type="text">
                </p>
                <button type="submit">Submit</button>
            </form>
        "#
                .to_string()
            }
        }
        None => "<p>Please <a href=\"/auth/login\">login</a> first in order submit your guess</p>"
            .to_string(),
    };

    let body = utils::load_view("day")?
        .replace("{{PAGE_TITLE}}", &format!("Day {day}"))
        .replace("{{DAY}}", &day.to_string())
        .replace("{{LOGIN_DYNA_BLOCK}}", &dynamic_form)
        .replace("{{DAY_IMG_SRC}}", &day_img_src)
        .replace("{{DAY_ORIGINAL_DATE}}", &picture_meta.original_date)
        .replace("{{GUESS_URL}}", &format!("/guess/{day}"))
        .replace("{{DAY_IMG_ALT}}", &format!("Image for day {day}"));

    Ok(body)
}
