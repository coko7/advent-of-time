use anyhow::{Result, bail};
use log::debug;
use std::{fs, path::Path};

use crate::models::user::User;

const DB_FILE_PATH: &str = "data/users.json";

pub fn initialize_database() -> Result<()> {
    if !Path::new(DB_FILE_PATH).exists() {
        fs::write(DB_FILE_PATH, "[]")?;
    }
    Ok(())
}

fn write_changes_to_database(users: &[User]) -> Result<()> {
    let json = serde_json::to_string(users)?;
    fs::write(DB_FILE_PATH, json)?;
    Ok(())
}

pub fn get_user_by_id(id: &str) -> Result<Option<User>> {
    Ok(get_all_users()?.iter().find(|u| u.id == id).cloned())
}

pub fn get_user_by_bearer(bearer_token: &str) -> Result<Option<User>> {
    Ok(get_all_users()?
        .iter()
        .find(|u| u.access_token == bearer_token)
        .cloned())
}

pub fn get_user_by_username(username: &str) -> Result<Option<User>> {
    Ok(get_all_users()?
        .iter()
        .find(|u| u.username == username)
        .cloned())
}

pub fn get_all_users() -> Result<Vec<User>> {
    let users_raw = fs::read_to_string(DB_FILE_PATH)?;
    let users = serde_json::from_str::<Vec<User>>(&users_raw)?;
    Ok(users)
}

pub fn create_user(user: User) -> Result<()> {
    let mut all_users = get_all_users()?;
    if let Some(existing_user) = all_users.iter().find(|u| u.id == user.id) {
        bail!(
            "user with ID `{}` already exists: {:?}",
            existing_user.id,
            existing_user
        );
    }

    debug!("created user: {:?}", user);
    all_users.push(user);
    write_changes_to_database(&all_users)
}

pub fn update_user(user: User) -> Result<()> {
    let mut all_users = get_all_users()?;
    if !all_users.iter().any(|u| u.id == user.id) {
        bail!("User does not exist: {:?}", user);
    }

    debug!("updated user: {:?}", user);
    all_users.retain(|u| u.id != user.id);
    all_users.push(user);
    write_changes_to_database(&all_users)
}

pub fn delete_user(user: &User) -> Result<()> {
    let mut all_users = get_all_users()?;
    all_users.retain(|u| u.id != user.id);
    debug!("deleted user: {:?}", user);
    write_changes_to_database(&all_users)
}
