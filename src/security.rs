use crate::{
    config::{Config, OAuth2Config},
    models::user::User,
};
use anyhow::{Result, bail};
use chrono::Local;

pub fn is_user_token_valid(user: &User) -> Result<bool> {
    Ok(user.access_token_expire_at > Local::now())
}

pub fn get_oauth2_provider_config(provider_name: &str) -> Result<OAuth2Config> {
    let config = Config::load_from_file()?;
    Ok(match provider_name {
        "discord" => config.oauth2.discord,
        "microsoft" => config.oauth2.microsoft,
        _ => bail!("unsupported IdP: {provider_name}"),
    })
}
