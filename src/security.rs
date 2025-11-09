use crate::{
    config::{Config, OAuth2Config},
    models::user::User,
};
use anyhow::{Result, bail};
use chrono::Utc;

pub fn has_access_token_expired(user: &User) -> bool {
    if let Some(expires_at) = user.access_token_expire_at {
        expires_at <= Utc::now()
    } else {
        true
    }
}

pub fn get_oauth2_provider_config(provider_name: &str) -> Result<OAuth2Config> {
    let config = Config::load_from_file()?;
    Ok(match provider_name {
        "discord" => config.oauth2.discord,
        "microsoft" => config.oauth2.microsoft,
        _ => bail!("unsupported IdP: {provider_name}"),
    })
}
