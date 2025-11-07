use anyhow::Result;
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder},
    router::RoutingData,
};
use serde::Serialize;
use serde_json::json;

use crate::{database::user_repository::UserRepository, models::user::User, utils};

#[derive(Debug, Serialize)]
struct LeaderboardUserEntry {
    pub username: String,
    pub guesses: usize,
    pub score: u32,
}

pub fn get_leaderboard_users(users: &[User]) -> Vec<LeaderboardUserEntry> {
    users
        .iter()
        .map(|u| LeaderboardUserEntry {
            username: u.username.to_owned(),
            guesses: u.guess_data.len(),
            score: u.get_total_score(),
        })
        .collect::<Vec<_>>()
}

pub fn get_leaderboard(
    _request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let users = UserRepository::get_all_users()?;

    let total_days = utils::get_current_day();
    let data = json!({
        "total_days": total_days,
        "users": get_leaderboard_users(&users)
    });
    let rendered = utils::render_view("leaderboard", &data)?;
    HttpResponseBuilder::new().set_html_body(&rendered).build()
}
