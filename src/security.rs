use crate::models::user::User;
use anyhow::Result;
use chrono::Local;

pub fn is_user_token_valid(user: &User) -> Result<bool> {
    Ok(user.access_token_expire_at > Local::now())
}
