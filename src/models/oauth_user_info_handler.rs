use anyhow::{Result, bail};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;

use crate::models::{oauth2_response::OAuth2Response, user::User};

pub trait OAuthUserInfoHandler<T>
where
    T: for<'a> Deserialize<'a>,
{
    fn user_info_url(&self) -> Result<String>;
    fn create_app_user(&self, oauth2_response: &OAuth2Response) -> Result<User>;
    fn fetch_user_info(&self, access_token: &str) -> Result<T> {
        let user_info_url = self.user_info_url()?;
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(user_info_url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, "coko7-aot-2025")
            .send()?;

        if !response.status().is_success() {
            bail!("failed to get user: {response:#?}");
        }

        let res = response.text()?;
        let res = serde_json::from_str::<T>(&res)?;
        Ok(res)
    }
}
