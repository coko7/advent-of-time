use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, FixedOffset, NaiveDateTime, Timelike, Utc};
use handlebars::Handlebars;
use rand::{SeedableRng, rngs::StdRng, seq::IndexedRandom};
use regex::Regex;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{fs, path::PathBuf};

pub type Day = u32;

const DICO_NOUNS_PATH: &str = "data/dictionaries/nouns.txt";
const DICO_ADJECTIVES_PATH: &str = "data/dictionaries/adjectives.txt";

pub fn str_to_u64seed(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

pub fn generate_username(seed: u64) -> Result<String> {
    let mut rng = StdRng::seed_from_u64(seed);

    let nouns = fs::read_to_string(DICO_NOUNS_PATH)?;
    let nouns = nouns.lines().collect::<Vec<_>>();

    let adjectives = fs::read_to_string(DICO_ADJECTIVES_PATH)?;
    let adjectives = adjectives.lines().collect::<Vec<_>>();

    let rnd_noun = nouns
        .choose(&mut rng)
        .context("nouns list should not be empty")?;
    let rnd_adj = adjectives
        .choose(&mut rng)
        .context("adjectives list should not be empty")?;

    let username = format!("{rnd_adj}-{rnd_noun}");
    Ok(username)
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

    let now = Utc::now();
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

// pub fn extract_time_from_image(img_path: &PathBuf) -> Result<(u32, u32)> {
//     rexiv2::initialize()?;
//     let metadata = Metadata::new_from_path(img_path)?;
//     println!("{:#?}", metadata.get_xmp_tags()?);
//     let raw_dt = metadata.get_tag_string("Exif.Photo.DateTimeOriginal")?;
//     println!("tag: {:#?}", raw_dt);
//     let datetime = NaiveDateTime::parse_from_str(&raw_dt, "%Y:%m:%d %H:%M:%S")?;
//     Ok((datetime.hour(), datetime.minute()))
// }

pub fn load_view(name: &str) -> Result<String> {
    let view_path = PathBuf::from(format!("src/views/{}.html", name));
    if view_path.exists() {
        return Ok(fs::read_to_string(view_path)?);
    }

    let view_path = PathBuf::from(format!("src/views/{}.hbs", name));
    Ok(fs::read_to_string(view_path)?)
}

pub fn render_view<T: Serialize>(name: &str, data: &T) -> Result<String> {
    let raw_view = load_view(name)?;
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string(name, raw_view)?;
    let rendered = handlebars.render(name, &data)?;
    Ok(rendered)
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

pub fn get_current_day() -> Day {
    let now = Utc::now();
    now.day()
}

pub fn is_picture_released(utc_now: DateTime<Utc>, picture_day: Day) -> bool {
    if picture_day > utc_now.day() {
        false
    } else if picture_day < utc_now.day() {
        true
    } else {
        is_time_after_6_am_cet(utc_now)
    }
}

fn is_time_after_6_am_cet(time: DateTime<Utc>) -> bool {
    // Define CET timezone offset (+01:00)
    let cet_offset = FixedOffset::east_opt(3600).unwrap();
    let cet_now: DateTime<FixedOffset> = time.with_timezone(&cet_offset);
    cet_now.hour() >= 6
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_str_to_u64seed() {
        let hash = str_to_u64seed("hello world!");
        assert_eq!(16348622334315420128, hash);
    }

    #[test]
    fn test_generate_username() {
        let username = generate_username(42).unwrap();
        assert_eq!("many-bun", username);
    }

    #[test]
    fn test_time_after_6_am_cet_false() {
        // 4:45:32 UTC is 5:45:32 CET
        let utc_time = Utc.with_ymd_and_hms(2025, 12, 15, 4, 45, 32).unwrap();
        assert!(!is_time_after_6_am_cet(utc_time))
    }

    #[test]
    fn test_time_after_6_am_cet_true() {
        // 5:01:00 UTC is 6:01:00 CET
        let utc_time = Utc.with_ymd_and_hms(2025, 12, 15, 5, 1, 0).unwrap();
        assert!(is_time_after_6_am_cet(utc_time))
    }

    #[test]
    fn test_is_picture_released_past_late_true() {
        let utc_time = Utc.with_ymd_and_hms(2025, 12, 15, 18, 0, 0).unwrap();
        let day = 10;
        assert!(is_picture_released(utc_time, day))
    }

    #[test]
    fn test_is_picture_released_past_early_true() {
        let utc_time = Utc.with_ymd_and_hms(2025, 12, 15, 3, 0, 0).unwrap();
        let day = 10;
        assert!(is_picture_released(utc_time, day))
    }

    #[test]
    fn test_is_picture_released_future_late_false() {
        let utc_time = Utc.with_ymd_and_hms(2025, 12, 15, 18, 0, 0).unwrap();
        let day = 20;
        assert!(!is_picture_released(utc_time, day))
    }

    #[test]
    fn test_is_picture_released_future_early_false() {
        let utc_time = Utc.with_ymd_and_hms(2025, 12, 15, 3, 0, 0).unwrap();
        let day = 20;
        assert!(!is_picture_released(utc_time, day))
    }
}
