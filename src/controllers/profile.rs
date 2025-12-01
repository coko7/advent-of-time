use anyhow::Result;
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder},
    router::RoutingData,
};
use rust_i18n::t;
use serde::Serialize;
use serde_json::json;
use std::cmp;

use crate::{
    http_helpers,
    models::user::User,
    utils::{self, Day},
};

pub fn get_me(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let user = match http_helpers::get_logged_in_user(request)? {
        Some(user) => user,
        None => return http_helpers::redirect("/auth/login"),
    };

    let data = json!({
        "username": &user.username,
        "account_name": &user.oauth_username,
        "days": get_user_guess_days(&user),
        "total_score": user.get_total_score()?,
        "i18n": I18n::from_request(request).unwrap(),
    });
    let rendered = utils::render_view("profile", &data)?;
    HttpResponseBuilder::new().set_html_body(&rendered).build()
}

#[derive(Debug, Serialize)]
struct UserGuessDay {
    pub day: Day,
    pub guessed: bool,
    pub time: String,
    pub points: u32,
}

fn get_user_guess_days(user: &User) -> Vec<UserGuessDay> {
    let current_day = cmp::min(25, utils::get_current_day());
    (1..=current_day)
        .map(|d| {
            user.guess_data.get(&d).map_or(
                UserGuessDay {
                    day: d,
                    guessed: false,
                    time: String::new(),
                    points: 0,
                },
                |guess| UserGuessDay {
                    day: d,
                    guessed: true,
                    time: guess.time(),
                    points: user.get_points(d).unwrap(),
                },
            )
        })
        .collect()
}

#[derive(Serialize)]
struct I18n {
    title: String,
    username: String,
    generated_username: String,
    account: String,
    day: String,
    guess: String,
    points: String,
    score: String,
}

impl I18n {
    fn from_request(request: &HttpRequest) -> Result<I18n> {
        let user_locale = http_helpers::get_user_locale(request)?.to_str();
        Ok(I18n {
            title: t!("profile.title", locale = user_locale).to_string(),
            username: t!("profile.username", locale = user_locale).to_string(),
            generated_username: t!("profile.generated_username", locale = user_locale).to_string(),
            account: t!("profile.account", locale = user_locale).to_string(),
            day: t!("profile.day", locale = user_locale).to_string(),
            guess: t!("profile.guess", locale = user_locale).to_string(),
            points: t!("profile.points", locale = user_locale).to_string(),
            score: t!("profile.score", locale = user_locale).to_string(),
        })
    }
}
