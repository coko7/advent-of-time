use anyhow::{Context, Result, bail};
use chrono::Utc;
use log::{debug, info, trace};
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder, response_status_codes::HttpStatusCode},
    router::RoutingData,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    database::{picture_meta_repository::PictureMetaRepository, user_repository::UserRepository},
    http_helpers::{self, bad_request, bad_request_msg},
    models::user::GuessData,
    utils,
};

#[derive(Serialize, Deserialize)]
pub struct SubmitGuessRequest {
    pub day: u32,
    pub guess: String,
}

pub fn post_guess(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let mut user = match http_helpers::get_logged_in_user(request)? {
        Some(user) => user,
        None => {
            return HttpResponseBuilder::new()
                .set_status(HttpStatusCode::Unauthorized)
                .build();
        }
    };

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

    if user.has_guessed(day) {
        return bad_request_msg("You have already guessed this day!");
    }

    let guess_value = request_data.guess;
    match parse_guess_value(&guess_value) {
        Ok(guess) => {
            let score = compute_score(day, guess)?;
            let guess_data = GuessData::new(guess, score, Utc::now());
            user.guess_data.insert(day, guess_data);
            UserRepository::update_user(user)?;

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
        .split_once(':')
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
    let daily_img_meta =
        PictureMetaRepository::get_picture(day)?.context("HEY where is my picture???")?;
    assert!(daily_img_meta.day() == day);

    let real_time_mins = daily_img_meta.hours()? * 60 + daily_img_meta.minutes()?;
    let guess_time_mins = guess.0 * 60 + guess.1;

    debug!("guessed time: {:02}:{:02}", guess.0, guess.1);
    debug!("real time: {}", daily_img_meta.time_taken);

    let diff_mins = (real_time_mins).abs_diff(guess_time_mins);
    debug!("diff in minutes: {diff_mins}");

    let points = utils::time_diff_to_points(diff_mins);
    debug!("points: {points}");
    Ok(points)
}
