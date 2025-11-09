use anyhow::Result;
use log::debug;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    models::{oauth2_response::OAuth2Response, user::User},
    utils,
};

#[derive(Deserialize, Debug)]
pub struct DiscordUserResponse {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub bot: Option<bool>,
    pub system: Option<bool>,
    pub mfa_enabled: Option<bool>,
    pub locale: Option<String>,
    pub verified: Option<bool>,
    pub email: Option<String>,
    pub flags: Option<u64>,
    pub premium_type: Option<u8>,
    pub public_flags: Option<u64>,
}

impl DiscordUserResponse {
    pub fn create_app_user(oauth2_response: &OAuth2Response) -> Result<User> {
        let user_info = Self::fetch_user_info(&oauth2_response.access_token)?;
        debug!("{user_info:#?}");

        let unique_hash = utils::str_to_u64seed(&user_info.id);
        let username = utils::generate_username(unique_hash)?;

        let mut user = User {
            id: user_info.id,
            username,
            oauth_username: user_info.username,
            guess_data: HashMap::new(),
            access_token: String::new(),
            access_token_expire_at: None,
            refresh_token: String::new(),
            oauth_provider: "discord".to_string(),
        };
        user.set_auth(oauth2_response)?;

        Ok(user)
    }

    fn fetch_user_info(access_token: &str) -> Result<Self> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get("https://discord.com/api/v10/users/@me")
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .send()?;

        let res = response.text()?;
        let res = serde_json::from_str::<Self>(&res)?;
        Ok(res)
    }
}
