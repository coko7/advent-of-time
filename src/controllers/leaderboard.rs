use anyhow::Result;
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder},
    router::RoutingData,
};
use serde::Serialize;
use serde_json::json;
use std::cmp;

use crate::{database::user_repository::UserRepository, models::user::User, utils};

#[derive(Debug, Serialize)]
struct LeaderboardUserEntry {
    pub username: String,
    pub guesses: usize,
    pub score: u32,
}

fn get_leaderboard_users(users: &[User]) -> Vec<LeaderboardUserEntry> {
    users
        .iter()
        .map(|u| LeaderboardUserEntry {
            username: u.username.to_owned(),
            guesses: u.guess_data.len(),
            score: u.get_total_score().unwrap(),
        })
        .collect::<Vec<_>>()
}

pub fn get_leaderboard(request: &HttpRequest, _routing_data: &RoutingData) -> Result<HttpResponse> {
    let mut users = UserRepository::get_all_users()?;
    users.sort_by_key(|u| cmp::Reverse(u.get_total_score().unwrap()));

    let total_days = utils::get_current_day();
    let data = json!({
        "total_days": total_days,
        "users": get_leaderboard_users(&users),
        "i18n": I18n::from_request(request)
    });
    let rendered = utils::render_view("leaderboard", &data)?;
    HttpResponseBuilder::new().set_html_body(&rendered).build()
}

#[derive(Serialize)]
struct I18n {
    title: String,
    user: String,
    guesses: String,
    score: String,
}

impl I18n {
    fn from_request(request: &HttpRequest) -> I18n {
        I18n {
            title: utils::load_i18n_for_user("leaderboard.title", request).unwrap(),
            user: utils::load_i18n_for_user("leaderboard.user", request).unwrap(),
            guesses: utils::load_i18n_for_user("leaderboard.guesses", request).unwrap(),
            score: utils::load_i18n_for_user("leaderboard.score", request).unwrap(),
        }
    }
}
