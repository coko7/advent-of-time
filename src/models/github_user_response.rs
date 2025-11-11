use anyhow::{Result, bail};
use chrono::{DateTime, Utc};
use log::debug;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    models::{oauth2_response::OAuth2Response, user::User},
    utils,
};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct GitHubUserResponse {
    pub login: String,
    pub id: u64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub site_admin: bool,
    pub name: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub location: Option<String>,
    pub email: Option<String>,
    pub hireable: Option<bool>,
    pub bio: Option<String>,
    pub twitter_username: Option<String>,
    pub public_repos: u64,
    pub public_gists: u64,
    pub followers: u64,
    pub following: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub private_gists: u64,
    pub total_private_repos: u64,
    pub owned_private_repos: u64,
    pub disk_usage: u64,
    pub collaborators: u64,
    pub two_factor_authentication: bool,
    pub plan: Option<Plan>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Plan {
    pub name: String,
    pub space: u64,
    pub private_repos: u64,
    pub collaborators: u64,
}

impl GitHubUserResponse {
    pub fn create_app_user(oauth2_response: &OAuth2Response) -> Result<User> {
        let user_info = Self::fetch_user_info(&oauth2_response.access_token)?;
        debug!("{user_info:#?}");

        let unique_hash = user_info.id;
        let username = utils::generate_username(unique_hash)?;

        let mut user = User {
            id: user_info.id.to_string(),
            username,
            oauth_username: user_info.login,
            guess_data: HashMap::new(),
            access_token: String::new(),
            access_token_expire_at: None,
            refresh_token: None,
            oauth_provider: "github".to_string(),
        };
        user.set_auth(oauth2_response)?;

        Ok(user)
    }

    fn fetch_user_info(access_token: &str) -> Result<Self> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get("https://api.github.com/user")
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, "coko7-aot-2025")
            .send()?;

        if !response.status().is_success() {
            bail!("failed to get user: {response:#?}");
        }

        let res = response.text()?;
        let res = serde_json::from_str::<Self>(&res)?;
        Ok(res)
    }
}
