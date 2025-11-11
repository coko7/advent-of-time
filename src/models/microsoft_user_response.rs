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

impl MicrosoftUserResponse {
    pub fn create_app_user(oauth2_response: &OAuth2Response) -> Result<User> {
        let user_info = Self::fetch_user_info(&oauth2_response.access_token)?;
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
        };
        user.set_auth(oauth2_response)?;

        Ok(user)
    }

    fn fetch_user_info(access_token: &str) -> Result<Self> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get("https://graph.microsoft.com/v1.0/me")
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .send()?;

        let res = response.text()?;
        let res = serde_json::from_str::<Self>(&res)?;
        Ok(res)
    }
}
