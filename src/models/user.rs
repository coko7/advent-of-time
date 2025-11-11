use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

use crate::{
    database::picture_meta_repository::PictureMetaRepository,
    models::oauth2_response::OAuth2Response,
    utils::{self, Day},
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub oauth_username: String,
    pub guess_data: HashMap<Day, GuessData>,
    pub access_token: String,
    pub access_token_expire_at: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
    pub oauth_provider: String,
}

impl User {
    pub fn has_guessed(&self, day: Day) -> bool {
        self.guess_data.contains_key(&day)
    }

    pub fn get_total_score(&self) -> Result<u32> {
        self.guess_data
            .keys()
            .try_fold(0, |acc, &day| Ok(acc + self.get_points(day)?))
    }

    pub fn get_points(&self, day: Day) -> Result<u32> {
        match self.guess_data.get(&day) {
            Some(data) => {
                // info!("received guess for day {day}: {guess:?}");
                let picture = PictureMetaRepository::get_picture(day)?
                    .context("picture should exist for guessed day")?;
                ensure!(picture.day() == day);
                utils::compute_score(&picture, data.hm)
            }

            None => Ok(0),
        }
    }

    pub fn set_auth(&mut self, oauth2_response: &OAuth2Response) -> Result<()> {
        let now = Utc::now();
        let at_expires_at = if let Some(expires_in) = oauth2_response.expires_in {
            let expires_in = expires_in - 30; // invalidate 30 seconds early
            Some(now + Duration::from_secs(expires_in))
        } else {
            None
        };

        self.access_token_expire_at = at_expires_at;
        self.access_token = oauth2_response.access_token.to_owned();
        self.refresh_token = oauth2_response.refresh_token.to_owned();
        Ok(())
    }

    pub fn clear_auth(&mut self) -> Result<()> {
        self.access_token_expire_at = None;
        self.access_token = String::new();
        self.refresh_token = None;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GuessData {
    /// The time at which the guess was made by that user
    pub taken_at: DateTime<Utc>,
    /// The hours/minutes keypair submitted by the user
    pub hm: (u32, u32),
}

impl GuessData {
    pub fn new(guess_hm: (u32, u32), taken_at: DateTime<Utc>) -> GuessData {
        GuessData {
            taken_at,
            hm: guess_hm,
        }
    }

    pub fn time(&self) -> String {
        format!("{:02}:{:02}", self.hm.0, self.hm.1)
    }
}
