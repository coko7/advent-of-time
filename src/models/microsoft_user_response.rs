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
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MicrosoftUserResponse {
    id: String,
    display_name: Option<String>,
    given_name: Option<String>,
    surname: Option<String>,
    user_principal_name: Option<String>,
    mail: Option<String>,
    job_title: Option<String>,
    mobile_phone: Option<String>,
    office_location: Option<String>,
    preferred_language: Option<String>,
}

pub struct MicrosoftUserInfoHandler;

impl OAuthUserInfoHandler<MicrosoftUserResponse> for MicrosoftUserInfoHandler {
    fn user_info_url(&self) -> Result<String> {
        let config = Config::get()?;
        Ok(config.oauth2.microsoft.user_info_url)
    }

    fn create_app_user(&self, oauth2_response: &OAuth2Response) -> Result<User> {
        let user_info = self.fetch_user_info(&oauth2_response.access_token)?;
        debug!("{user_info:#?}");

        let unique_hash = utils::str_to_u64seed(&user_info.id);
        let username = utils::generate_username(unique_hash)?;

        let mut user = User {
            id: user_info.id,
            username,
            oauth_username: user_info.user_principal_name.unwrap(),
            guess_data: HashMap::new(),
            access_token: String::new(),
            access_token_expire_at: None,
            refresh_token: None,
            oauth_provider: "microsoft".to_string(),
            hidden: false,
        };
        user.set_auth(oauth2_response)?;

        Ok(user)
    }
}
