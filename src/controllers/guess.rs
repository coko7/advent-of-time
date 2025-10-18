use anyhow::{Context, Result, bail};
use chrono::{Datelike, Local, TimeZone};
use log::{debug, info, trace};
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder, response_status_codes::HttpStatusCode},
    router::RoutingData,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils;

#[derive(Serialize, Deserialize)]
pub struct SubmitGuessRequest {
    pub day: u32,
    pub guess: String,
}

fn bad_request() -> Result<HttpResponse> {
    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::BadRequest)
        .build()
}

fn bad_request_msg(message: &str) -> Result<HttpResponse> {
    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::BadRequest)
        .set_html_body(message)
        .build()
}

pub fn post_guess(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let body = match request.get_str_body() {
        Ok(body) => body,
        Err(e) => {
            trace!("invalid guess format: {:?}", request.body);
            debug!("failed to get str body: {e}");
            return bad_request();
        }
    };

    let request_data = match serde_json::from_str::<SubmitGuessRequest>(&body) {
        Ok(value) => value,
        Err(e) => {
            trace!("invalid guess json format: {:?}", &body);
            debug!("failed to create guess struct from json: {e}");
            return bad_request();
        }
    };

    let day = request_data.day;
    if !utils::is_day_valid(day) {
        debug!("invalid day requested: {day}");
        return bad_request();
    }

    let guess_value = request_data.guess;
    match parse_guess_value(&guess_value) {
        Ok(guess) => {
            let score = compute_score(day, guess)?;

            HttpResponseBuilder::new()
                .set_json_body(&json!({"points": score}))?
                .build()
        }
        Err(err) => {
            let error_json = json!({
                "error": err.to_string()
            });

            HttpResponseBuilder::new()
                .set_status(HttpStatusCode::BadRequest)
                .set_json_body(&error_json)?
                .build()
        }
    }
}

fn parse_guess_value(guess: &str) -> Result<(u32, u32)> {
    if guess.len() > 5 {
        bail!("guess value does not denote a valid time");
    }

    let parts = guess
        .split_once('h')
        .context("guess value should contain 'h'")?;
    let hour: u32 = parts.0.parse()?;
    let minutes: u32 = parts.1.parse()?;

    if hour > 23 {
        bail!("invalid hour format: {hour}");
    }

    if minutes > 59 {
        bail!("invalid minute format: {hour}");
    }

    Ok((hour, minutes))
}

fn compute_score(day: u32, guess: (u32, u32)) -> Result<u32> {
    info!("received guess for day {day}: {guess:?}");
    let daily_img_meta = utils::load_img_meta(day)?;
    assert!(daily_img_meta.taken_at.day() == day);

    let real_dt = daily_img_meta.taken_at;
    let guess_dt = Local
        .with_ymd_and_hms(
            real_dt.year(),
            real_dt.month(),
            real_dt.day(),
            guess.0,
            guess.1,
            0,
        )
        .unwrap();

    debug!("guessed time: {guess_dt:?}");
    debug!("real time: {real_dt:?}");

    let diff = real_dt.signed_duration_since(guess_dt);
    let diff_minutes = diff.num_minutes().unsigned_abs();
    debug!("diff in minutes: {diff_minutes}");

    let points = utils::time_diff_to_points(diff_minutes);
    debug!("points: {points}");
    Ok(points)
}
