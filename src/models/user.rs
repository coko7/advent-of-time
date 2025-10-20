use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::utils::Day;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub guess_data: HashMap<Day, GuessData>,
    pub access_token: String,
    pub access_token_expire_at: DateTime<Utc>,
    pub refresh_token: String,
}

impl User {
    pub fn has_guessed(&self, day: Day) -> bool {
        self.guess_data.contains_key(&day)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GuessData {
    /// The time at which the guess was made by that user
    pub taken_at: DateTime<Utc>,
    /// The hours/minutes keypair submitted by the user
    pub hm: (u32, u32),
    /// Points associated to this guess (computed field)
    pub points: u32,
}

impl GuessData {
    pub fn new(guess_hm: (u32, u32), points: u32, taken_at: DateTime<Utc>) -> GuessData {
        GuessData {
            taken_at,
            hm: guess_hm,
            points,
        }
    }
}
