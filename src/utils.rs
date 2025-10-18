use anyhow::{Result, anyhow};
use chrono::{DateTime, Datelike, Local, TimeZone, Utc};
use log::{debug, error};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    hash::Hash,
    path::{Component, Path, PathBuf},
};

use crate::models::aot_image_meta::AotImageMeta;

const AOT_PICS_DIR: &str = "data/day-pics/";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct User {
    pub id: usize,
    pub firstname: String,
    pub lastname: String,
    pub title: String,
    pub image_url: String,
    pub born: String,
    pub died: Option<String>,
}

impl User {
    pub fn name(&self) -> String {
        format!("{} {}", self.firstname, self.lastname)
    }
}

pub fn is_safe_relative_subpath(path: &Path) -> bool {
    !path.is_absolute() && path.components().all(|comp| comp != Component::ParentDir)
}

pub fn get_aot_pics_dir() -> PathBuf {
    PathBuf::from(AOT_PICS_DIR)
}

pub fn markdown_to_html(content: &str) -> Result<String> {
    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")?;
    let result = link_regex
        .replace_all(content, r#"<a href="$2">$1</a>"#)
        .to_string();

    let bold_regex = Regex::new(r"\*\*(.*?)\*\*")?;
    let result = bold_regex.replace_all(&result, "<b>$1</b>").to_string();

    let italic_regex = Regex::new(r"\*(.*?)\*")?;
    let result = italic_regex.replace_all(&result, "<i>$1</i>").to_string();

    let code_regex = Regex::new(r"`(.*?)`")?;
    let result = code_regex
        .replace_all(&result, "<code>$1</code>")
        .to_string();

    Ok(result)
}

pub fn sanitize_filename(filename: &str) -> Result<OsString> {
    match Path::new(filename).file_name() {
        Some(filename) => Ok(filename.to_owned()),
        None => Err(anyhow!("failed to get filename")),
    }
}

pub fn is_day_valid(day: u32) -> bool {
    if day < 1 || day > 25 {
        return false;
    }

    let now = Local::now();
    let now_day = now.day();
    let month = now.month();

    // if month != 12 {
    //     return false;
    // }

    if day > now_day {
        return false;
    }

    return true;
}

pub fn get_day_img_path(day: u32) -> Result<PathBuf> {
    let picture_filename = format!("{day}.jpg");
    Ok(get_aot_pics_dir().join(picture_filename))
}

pub fn load_view(name: &str) -> Result<String> {
    let view_path = format!("src/views/{}.html", name);
    let content = fs::read_to_string(view_path)?;
    Ok(content)
}

pub fn sanitize_user_input(value: &str) -> String {
    value.replace("<", "&lt;").replace(">", "&gt;")
}

pub fn get_files_in_directory(path: &PathBuf) -> Result<Vec<PathBuf>> {
    let entries = fs::read_dir(path)?;
    let mut files: Vec<_> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() { Some(path) } else { None }
        })
        .collect();

    // sort file list by create/modify date in reverse orde (newest first)
    files.sort_by(|a, b| {
        let a_meta = a.metadata().unwrap();
        let b_meta = b.metadata().unwrap();

        let a_time = a_meta
            .created()
            .unwrap_or_else(|_| a_meta.modified().unwrap());
        let b_time = b_meta
            .created()
            .unwrap_or_else(|_| b_meta.modified().unwrap());

        a_time.cmp(&b_time).reverse()
    });

    Ok(files)
}

pub fn time_diff_to_points(diff_minutes: u32) -> u32 {
    match diff_minutes {
        0 => 25,
        1..15 => 19,
        15..30 => 14,
        30..60 => 10,
        60..120 => 7,  // 2 hours
        120..180 => 5, // 3 hours
        180..300 => 4, // 5 hours
        300..420 => 3, // 7 hours
        420..600 => 2, // 10 hours
        _ => 1,        // more than 10 hours
    }
}

pub fn load_img_meta(day: u32) -> Result<AotImageMeta> {
    error!("NOT yet implemented");
    let dt = Utc.with_ymd_and_hms(2025, 12, day, 12, 38, 27).unwrap();
    Ok(AotImageMeta {
        day: day,
        taken_at: dt,
        location: Some("Stockholm".to_owned()),
    })
}
