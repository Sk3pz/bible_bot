use crate::hey;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct GuildFile {
    daily_verse_channel: Option<u64>,
    reading_schedule_channel: Option<u64>,
}

pub struct GuildSettings {
    pub id: GuildId,
    pub file: GuildFile,
}

impl GuildSettings {
    pub fn new(guild_id: &GuildId) -> Self {
        Self {
            id: guild_id.clone(),
            file: GuildFile {
                daily_verse_channel: None,
                reading_schedule_channel: None,
            },
        }
    }

    pub fn get_guild_files() -> Vec<GuildSettings> {
        // loop through all the files (`./guilds/{}.json`)
        // and get the guild IDs from the filenames
        let mut guild_files = Vec::new();
        let paths = fs::read_dir("./guilds/").unwrap();

        // loop through the files
        for path in paths {
            // get the file extension
            let path = path.unwrap().path();
            if let Some(extension) = path.extension() {
                // ensure its a json file
                if extension == "json" {
                    // get the guild ID from the filename
                    if let Some(filename) = path.file_stem() {
                        // parse the guild ID
                        if let Some(guild_id_str) = filename.to_str() {
                            if let Ok(guild_id_num) = guild_id_str.parse::<u64>() {
                                // create a GuildId
                                let guild_id = GuildId::new(guild_id_num);
                                // get the guild file
                                let guild_file = GuildSettings::get(&guild_id);
                                guild_files.push(guild_file);
                            }
                        }
                    }
                }
            }
        }

        guild_files
    }

    pub fn get(id: &GuildId) -> Self {
        let raw_path = format!("./guilds/{}.json", id);
        let path = std::path::Path::new(&raw_path);

        if !path.exists() {
            Self::generate(id);
            return Self::new(id);
        }

        let Ok(data) = fs::read_to_string(path) else {
            Self::generate(id);
            return Self::new(id);
        };

        let guildfile: GuildFile = serde_json::from_str(data.as_str())
            .expect(format!("failed to deserialize guild data with ID {}", id).as_str());

        Self {
            id: id.clone(),
            file: guildfile,
        }
    }

    fn generate(id: &GuildId) {
        let raw_path = format!("./guilds/{}.json", id.get());
        let path = Path::new(raw_path.as_str());

        if path.exists() {
            hey!("Guild data already exists: {}", id);
            return;
        };

        let Ok(mut file) = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .append(false)
            .open(path)
        else {
            hey!("Failed to get file for guild data: {}", id);
            return;
        };

        let default_file = Self::new(id);

        let Ok(data) = serde_json::to_string(&default_file.file) else {
            hey!("Failed to serialize guild data: {}", id.clone());
            return;
        };

        //let default = "{\"level\":1,\"prestige\":1,\"ascension\":0,\"bananas\":0}".to_string();

        if let Err(e) = write!(file, "{}", data) {
            hey!("Failed to write to file for guild {}: {}", id, e);
        }
    }

    fn reload(&mut self) {
        *self = Self::get(&self.id);
    }

    fn update(&self) {
        let raw_path = format!("./guilds/{}.json", self.id.get());
        let path = Path::new(raw_path.as_str());

        if !path.exists() {
            Self::generate(&self.id);
        };

        let Ok(mut file) = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .append(false)
            .truncate(true)
            .open(path)
        else {
            hey!("Failed to get file for guild data: {}", &self.id);
            return;
        };

        let Ok(data) = serde_json::to_string(&self.file) else {
            hey!("Failed to serialize guild data: {}", &self.id);
            return;
        };

        if let Err(e) = write!(file, "{}", data) {
            hey!("Failed to write to file for guild {}: {}", &self.id, e);
        }
    }

    pub fn get_daily_verse_channel(&self) -> Option<ChannelId> {
        let id = self.file.daily_verse_channel?;
        Some(ChannelId::from(id))
    }

    pub fn get_daily_verse_channel_as_mut(&mut self) -> Option<ChannelId> {
        self.reload();
        let id = self.file.daily_verse_channel?;
        Some(ChannelId::from(id))
    }

    pub fn get_reading_schedule_channel(&self) -> Option<ChannelId> {
        let id = self.file.reading_schedule_channel?;
        Some(ChannelId::from(id))
    }

    pub fn get_reading_schedule_channel_as_mut(&mut self) -> Option<ChannelId> {
        self.reload();
        let id = self.file.reading_schedule_channel?;
        Some(ChannelId::from(id))
    }

    pub fn set_daily_verse_channel(&mut self, channel_id: ChannelId) {
        self.reload();
        self.file.daily_verse_channel = Some(channel_id.get());
        self.update();
    }

    pub fn set_reading_schedule_channel(&mut self, channel_id: ChannelId) {
        self.reload();
        self.file.reading_schedule_channel = Some(channel_id.get());
        self.update();
    }

    pub fn clear_daily_verse_channel(&mut self) {
        self.reload();
        self.file.daily_verse_channel = None;
        self.update();
    }

    pub fn clear_reading_schedule_channel(&mut self) {
        self.reload();
        self.file.reading_schedule_channel = None;
        self.update();
    }

    pub fn clear_channel_by_id(&mut self, channel_id: u64) {
        self.reload();
        if let Some(id) = self.file.daily_verse_channel {
            if id == channel_id {
                self.file.daily_verse_channel = None;
            }
        }
        if let Some(id) = self.file.reading_schedule_channel {
            if id == channel_id {
                self.file.reading_schedule_channel = None;
            }
        }
        self.update();
    }
}
