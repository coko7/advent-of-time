use anyhow::{Result, bail};
use log::debug;
use std::{fs, path::Path};

use crate::{models::picture::Picture, utils::Day};

const DB_FILE_PATH: &str = "data/pictures.json";

pub struct PictureMetaRepository;

impl PictureMetaRepository {
    pub fn initialize_database() -> Result<()> {
        if !Path::new(DB_FILE_PATH).exists() {
            fs::write(DB_FILE_PATH, "[]")?;
        }
        Ok(())
    }

    fn write_changes_to_database(pictures: &[Picture]) -> Result<()> {
        let json = serde_json::to_string(pictures)?;
        fs::write(DB_FILE_PATH, json)?;
        Ok(())
    }

    pub fn get_picture(day: Day) -> Result<Option<Picture>> {
        Ok(Self::get_all_pictures()?
            .iter()
            .find(|p| p.id == day)
            .cloned())
    }

    pub fn get_all_pictures() -> Result<Vec<Picture>> {
        let pictures_raw = fs::read_to_string(DB_FILE_PATH)?;
        let pictures = serde_json::from_str::<Vec<Picture>>(&pictures_raw)?;
        Ok(pictures)
    }

    pub fn create_picture(picture: Picture) -> Result<()> {
        let mut all_pictures = Self::get_all_pictures()?;
        if let Some(existing_picture) = all_pictures.iter().find(|u| u.id == picture.id) {
            bail!(
                "picture for day `{}` already exists: {:?}",
                existing_picture.id,
                existing_picture
            );
        }

        debug!("created picture: {:?}", picture);
        all_pictures.push(picture);
        Self::write_changes_to_database(&all_pictures)
    }

    pub fn update_picture(picture: Picture) -> Result<()> {
        let mut all_pictures = Self::get_all_pictures()?;
        if !all_pictures.iter().any(|u| u.id == picture.id) {
            bail!("Picture does not exist: {:?}", picture);
        }

        debug!("updated picture: {:?}", picture);
        all_pictures.retain(|u| u.id != picture.id);
        all_pictures.push(picture);
        Self::write_changes_to_database(&all_pictures)
    }

    pub fn delete_picture(picture: &Picture) -> Result<()> {
        let mut all_pictures = Self::get_all_pictures()?;
        all_pictures.retain(|u| u.id != picture.id);
        debug!("deleted picture: {:?}", picture);
        Self::write_changes_to_database(&all_pictures)
    }
}
