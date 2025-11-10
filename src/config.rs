use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use bible_lib::Translation;
use serde::{Deserialize, Serialize};

use crate::{hey, DEFAULT_TRANSLATION};

#[derive(Serialize, Deserialize, Debug, Clone)]
enum SerializableTranslation {
    AmericanStandard,
    AmericanKingJames,
    KingJames,
    EnglishRevised,
    Custom { name: String, path: String },
}

impl Into<Translation> for SerializableTranslation {
    fn into(self) -> Translation {
        match self {
            SerializableTranslation::AmericanStandard => Translation::AmericanStandard,
            SerializableTranslation::AmericanKingJames => Translation::AmericanKingJames,
            SerializableTranslation::KingJames => Translation::KingJames,
            SerializableTranslation::EnglishRevised => Translation::EnglishedRevised,
            SerializableTranslation::Custom { name, path } => Translation::Custom { name, path },
        }
    }
}

impl From<Translation> for SerializableTranslation {
    fn from(t: Translation) -> Self {
        match t {
            Translation::AmericanStandard => SerializableTranslation::AmericanStandard,
            Translation::AmericanKingJames => SerializableTranslation::AmericanKingJames,
            Translation::KingJames => SerializableTranslation::KingJames,
            Translation::EnglishedRevised => SerializableTranslation::EnglishRevised,
            Translation::Custom { name, path } => SerializableTranslation::Custom { name, path },
        }
    }
}

impl From<&Translation> for SerializableTranslation {
    fn from(t: &Translation) -> Self {
        match t {
            Translation::AmericanStandard => SerializableTranslation::AmericanStandard,
            Translation::AmericanKingJames => SerializableTranslation::AmericanKingJames,
            Translation::KingJames => SerializableTranslation::KingJames,
            Translation::EnglishedRevised => SerializableTranslation::EnglishRevised,
            Translation::Custom { name, path } => SerializableTranslation::Custom {
                name: name.clone(),
                path: path.clone(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSettings {
    translation: SerializableTranslation,
}

impl ConfigSettings {
    pub fn new(translation: &Translation) -> Self {
        Self {
            translation: SerializableTranslation::from(translation),
        }
    }

    pub fn get() -> Self {
        let raw_path = format!("./config.json");
        let path = std::path::Path::new(&raw_path);

        if !path.exists() {
            Self::generate();
            return Self::new(&DEFAULT_TRANSLATION);
        }

        let Ok(data) = fs::read_to_string(path) else {
            Self::generate();
            return Self::new(&DEFAULT_TRANSLATION);
        };

        let cfg: ConfigSettings = serde_json::from_str(data.as_str())
            .expect(format!("failed to deserialize config data").as_str());

        Self {
            translation: cfg.translation,
        }
    }

    fn generate() {
        let raw_path = format!("./config.json");
        let path = Path::new(raw_path.as_str());

        if path.exists() {
            hey!("Config data already exists");
            return;
        };

        let Ok(mut file) = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .append(false)
            .open(path)
        else {
            hey!("Failed to get file for config file.");
            return;
        };

        let default_file = Self::new(&DEFAULT_TRANSLATION);

        let Ok(data) = serde_json::to_string(&default_file) else {
            hey!("Failed to serialize config data.");
            return;
        };

        if let Err(e) = write!(file, "{}", data) {
            hey!("Failed to write to file for config: {}", e);
        }
    }

    pub fn get_translation(&self) -> Translation {
        self.translation.clone().into()
    }
}
