use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

type Day = u32;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub user_id: String,
    pub guess_data: HashMap<Day, GuessData>,
    pub access_token: String,
    pub access_token_expire_at: DateTime<Utc>,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GuessData {
    /// The time at which the guess was made by that user
    pub taken_at: DateTime<Utc>,
    /// The hours/minutes keypair submitted by the user
    pub hm: (u32, u32),
    /// Points associated to this guess (computed field)
    pub points: u32,
}
