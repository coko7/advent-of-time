use anyhow::Result;
use chrono::{Datelike, Local, TimeZone, Utc};
use log::error;
use regex::Regex;
use std::{fs, path::PathBuf};

use crate::models::aot_image_meta::AotImageMeta;

const AOT_PICS_DIR: &str = "data/day-pics/";

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

pub fn is_day_valid(day: u32) -> bool {
    if !(1..=25).contains(&day) {
        return false;
    }

    let now = Local::now();
    let now_day = now.day();
    let _month = now.month();

    // if month != 12 {
    //     return false;
    // }

    if day > now_day {
        return false;
    }

    true
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

pub fn time_diff_to_points(diff_minutes: u64) -> u32 {
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
        day,
        taken_at: dt,
        location: Some("Stockholm".to_owned()),
    })
}
