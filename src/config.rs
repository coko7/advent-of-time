use anyhow::Result;
use std::fs;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub hostname: String,
    pub oauth2: OAuth2Providers,
}

impl Config {
    pub fn load_from_file() -> Result<Config> {
        let config_str = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug)]
pub struct OAuth2Providers {
    pub discord: OAuth2Config,
    pub microsoft: OAuth2Config,
}

#[derive(Deserialize, Debug)]
pub struct OAuth2Config {
    pub enabled: bool,
    pub authorize_url: String,
    pub token_url: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub secret: String,
}
