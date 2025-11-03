use anyhow::Result;
use chrono::Local;
use log::debug;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

use crate::{
    models::{oauth2_response::OAuth2Response, user::User},
    utils,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
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

        let now = Local::now();
        let expires_in = oauth2_response.expires_in - 30; // invalidate 30 seconds early
        let at_expires_at = now + Duration::from_secs(expires_in);

        let unique_hash = utils::str_to_u64seed(&user_info.id);
        let username = utils::generate_username(unique_hash)?;

        Ok(User {
            id: user_info.id,
            username,
            oauth_username: user_info.user_principal_name.unwrap(),
            guess_data: HashMap::new(),
            access_token: oauth2_response.access_token.to_owned(),
            access_token_expire_at: at_expires_at.into(),
            refresh_token: oauth2_response.refresh_token.to_owned(),
        })
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
