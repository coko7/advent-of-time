use std::cmp;

use anyhow::Result;
use rtfw_http::{
    http::{HttpRequest, HttpResponse, HttpResponseBuilder},
    router::RoutingData,
};

use crate::{database, models::user::User, utils};

fn generate_leaderboard_table(users: &mut [User]) -> String {
    users.sort_by_key(|u| cmp::Reverse(u.get_total_score()));
    let mut res = String::from(
        r#"
                <table>
                    <tr>
                        <th>User</th>
                        <th>Guesses</th>
                        <th>Score</th>
                    </tr>
        "#,
    );

    let total_days = cmp::min(utils::get_current_day(), 25);
    for user in users {
        let username = user.username.clone();
        let guesses = user.guess_data.len();
        let score = user.get_total_score();

        let line = format!(
            r#"
            <tr>
                <td>{username}</td>
                <td>{guesses} / {total_days}</td>
                <td>{score} ⭐</td>
            </tr>
        "#
        );
        res.push_str(&line);
    }

    res.push_str("</table>");
    res
}

pub fn get_leaderboard(
    _request: &HttpRequest,
    _routing_data: &RoutingData,
) -> Result<HttpResponse> {
    let mut users = database::get_all_users()?;
    let leaderboard = generate_leaderboard_table(&mut users);

    let body = utils::load_view("leaderboard")?.replace("{{LEADERBOARD_BLOCK}}", &leaderboard);
    HttpResponseBuilder::new().set_html_body(&body).build()
}
