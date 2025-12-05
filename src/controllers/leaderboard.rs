use anyhow::Result;
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder},
    router::RoutingData,
};
use rust_i18n::t;
use serde::Serialize;
use serde_json::json;
use std::cmp;

use crate::{database::user_repository::UserRepository, http_helpers, models::user::User, utils};

#[derive(Debug, Serialize)]
struct LeaderboardUserEntry {
    pub rank: String,
    pub username: String,
    pub guesses: usize,
    pub score: u32,
    pub accuracy: Option<u32>,
    pub hidden: bool,
}

fn get_leaderboard_users(users: &[User]) -> Vec<LeaderboardUserEntry> {
    users
        .iter()
        .enumerate()
        .map(|(rank, user)| {
            let score = user.get_total_score().unwrap();
            let guesses = user.guess_data.len();
            let accuracy = if guesses > 0 {
                Some(score / guesses as u32)
            } else {
                None
            };

            LeaderboardUserEntry {
                rank: (rank + 1).to_string(),
                username: user.username.to_owned(),
                guesses,
                accuracy,
                score,
                hidden: user.hidden || guesses == 0,
            }
        })
        .collect::<Vec<_>>()
}

pub fn get_leaderboard(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let mut users = UserRepository::get_all_users()?
        .iter()
        .filter(|u| !u.hidden)
        .cloned()
        .collect::<Vec<_>>();
    users.sort_by_key(|u| cmp::Reverse(u.get_total_score().unwrap()));

    let total_days = utils::get_current_day();
    let data = json!({
        "total_days": total_days,
        "users": get_leaderboard_users(&users),
        "i18n": I18n::from_request(request).unwrap()
    });
    let rendered = utils::render_view("leaderboard", &data)?;
    HttpResponseBuilder::new().set_html_body(&rendered).build()
}

#[derive(Serialize)]
struct I18n {
    title: String,
    rank: String,
    user: String,
    guesses: String,
    score: String,
    accuracy: String,
    text_max_score: String,
    check_point_system: String,
}

impl I18n {
    fn from_request(request: &HttpRequest) -> Result<I18n> {
        let user_locale = http_helpers::get_user_locale(request)?.to_str();
        Ok(I18n {
            title: t!("leaderboard.title", locale = user_locale).to_string(),
            rank: t!("leaderboard.rank", locale = user_locale).to_string(),
            user: t!("leaderboard.user", locale = user_locale).to_string(),
            guesses: t!("leaderboard.guesses", locale = user_locale).to_string(),
            score: t!("leaderboard.score", locale = user_locale).to_string(),
            accuracy: t!("leaderboard.accuracy", locale = user_locale).to_string(),
            text_max_score: t!("leaderboard.text_max_score", locale = user_locale).to_string(),
            check_point_system: t!("check_point_system", locale = user_locale).to_string(),
        })
    }
}
