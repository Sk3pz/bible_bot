use std::fs;
use std::io::Write;
use std::{fs::OpenOptions, path::Path};

use bible_lib::{Bible, BibleLookup};

use crate::hey;

const PATH: &str = "./data/daily_verse.json";

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct LookupStore {
    pub book: String,
    pub chapter: u32,
    pub verse: u32,
}

impl From<BibleLookup> for LookupStore {
    fn from(lookup: BibleLookup) -> Self {
        Self {
            book: lookup.book,
            chapter: lookup.chapter,
            verse: lookup.verse,
        }
    }
}

impl Into<BibleLookup> for LookupStore {
    fn into(self) -> BibleLookup {
        BibleLookup {
            book: self.book,
            chapter: self.chapter,
            verse: self.verse,
            thru_verse: None,
        }
    }
}

pub struct DailyVerseHandler {
    lookup: LookupStore,
}

impl DailyVerseHandler {
    pub fn new(bible: &Bible) -> Self {
        Self {
            lookup: bible.random_verse().into(),
        }
    }

    pub fn get(bible: &Bible) -> Self {
        let raw_path = format!("{}", PATH);
        let path = std::path::Path::new(&raw_path);

        if !path.exists() {
            if let Some(new) = Self::generate(bible) {
                return new;
            } else {
                return Self::new(bible);
            }
        }

        let Ok(data) = fs::read_to_string(path) else {
            if let Some(new) = Self::generate(bible) {
                return new;
            } else {
                return Self::new(bible);
            }
        };

        let lookup: LookupStore;
        if let Ok(v) = serde_json::from_str(data.as_str()) {
            lookup = v;
        } else {
            return Self::new(bible);
        };

        Self { lookup }
    }

    pub fn generate(bible: &Bible) -> Option<Self> {
        let raw_path = format!("{}", PATH);
        let path = Path::new(raw_path.as_str());

        if path.exists() {
            hey!("Verse data already exists!");
            return None;
        };

        let Ok(mut file) = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .append(false)
            .open(path)
        else {
            hey!("Failed to get verse data file!");
            return None;
        };

        let default_file = Self::new(bible);

        let Ok(data) = serde_json::to_string(&default_file.lookup) else {
            hey!("Failed to serialize verse data");
            return None;
        };

        //let default = "{\"level\":1,\"prestige\":1,\"ascension\":0,\"bananas\":0}".to_string();

        if let Err(e) = write!(file, "{}", data) {
            hey!("Failed to write to file for verse data: {}", e);
        }

        Some(default_file)
    }

    fn update(&self, bible: &Bible) {
        let raw_path = format!("{}", PATH);
        let path = Path::new(raw_path.as_str());

        if !path.exists() {
            Self::generate(bible);
        };

        let Ok(mut file) = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .append(false)
            .truncate(true)
            .open(path)
        else {
            hey!("Failed to get file for verse data");
            return;
        };

        let Ok(data) = serde_json::to_string(&self.lookup) else {
            hey!("Failed to serialize verse data");
            return;
        };

        if let Err(e) = write!(file, "{}", data) {
            hey!("Failed to write to file for verse data: {}", e);
        }
    }

    pub fn get_verse(&self) -> BibleLookup {
        self.lookup.clone().into()
    }

    pub fn set_new_verse(&mut self, bible: &Bible) {
        let new_lookup: LookupStore = bible.random_verse().into();
        self.lookup = new_lookup;
        self.update(bible);
    }

    pub fn set_custom_verse(&mut self, custom: BibleLookup, bible: &Bible) {
        let new_lookup: LookupStore = custom.into();
        self.lookup = new_lookup;
        self.update(bible);
    }
}
