use anyhow::Result;
use serde::Deserialize;

pub const CONFIG_RAW: &str = include_str!("../config.toml");

#[derive(Deserialize)]
pub struct Config {
    pub hostname: String,
    pub dev_mode: bool,
    pub oauth2: OAuth2Providers,
    pub score: ScoreConfig,
}

impl Config {
    pub fn get() -> Result<Config> {
        let config: Config = toml::from_str(CONFIG_RAW)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug)]
pub struct ScoreConfig {
    pub max_reward: f64,
    pub exponent: f64,
    pub divider: u32,
}

#[derive(Deserialize, Debug)]
pub struct OAuth2Providers {
    pub discord: OAuth2Config,
    pub microsoft: OAuth2Config,
    pub github: OAuth2Config,
}

#[derive(Deserialize, Debug)]
pub struct OAuth2Config {
    pub enabled: bool,
    pub authorize_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub secret: String,
}
