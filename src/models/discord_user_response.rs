use anyhow::Result;
use log::debug;
use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    config::Config,
    models::{
        oauth_user_info_handler::OAuthUserInfoHandler, oauth2_response::OAuth2Response, user::User,
    },
    utils,
};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
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

pub struct DiscordUserInfoHandler;

impl OAuthUserInfoHandler<DiscordUserResponse> for DiscordUserInfoHandler {
    fn user_info_url(&self) -> Result<String> {
        let config = Config::get()?;
        Ok(config.oauth2.discord.user_info_url)
    }

    fn create_app_user(&self, oauth2_response: &OAuth2Response) -> Result<User> {
        let user_info = self.fetch_user_info(&oauth2_response.access_token)?;
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
            refresh_token: None,
            oauth_provider: "discord".to_string(),
            hidden: false,
        };
        user.set_auth(oauth2_response)?;

        Ok(user)
    }
}
