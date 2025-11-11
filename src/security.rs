use crate::{
    config::{Config, OAuth2Config},
    models::user::User,
};
use anyhow::{Context, Result, bail};
use chrono::Utc;

pub fn has_access_token_expired(user: &User) -> Result<bool> {
    let expires_at = user
        .access_token_expire_at
        .context("expire time should be set to call this method")?;
    Ok(expires_at <= Utc::now())
}

pub fn get_oauth2_provider_config(provider_name: &str) -> Result<OAuth2Config> {
    let config = Config::load_from_file()?;
    Ok(match provider_name {
        "discord" => config.oauth2.discord,
        "microsoft" => config.oauth2.microsoft,
        "github" => config.oauth2.github,
        _ => bail!("unsupported IdP: {provider_name}"),
    })
}
