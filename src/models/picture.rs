use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::utils::Day;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Picture {
    pub id: Day,
    pub path: PathBuf,
    pub original_date: String,
    pub time_taken: String,
    pub location: Option<String>,
}

impl Picture {
    pub fn day(&self) -> Day {
        self.id
    }

    pub fn get_full_path(&self) -> PathBuf {
        PathBuf::from("data").join(&self.path)
    }

    pub fn hours(&self) -> Result<u32> {
        Ok(self
            .time_taken
            .split_once(':')
            .context("should be two components")?
            .0
            .parse::<u32>()?)
    }

    pub fn minutes(&self) -> Result<u32> {
        Ok(self
            .time_taken
            .split_once(':')
            .context("should be two components")?
            .1
            .parse::<u32>()?)
    }
}
