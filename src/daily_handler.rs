use std::sync::{Arc, Mutex};

use bible_lib::{Bible, BibleLookup};
use chrono::{Local, Timelike};
use serenity::all::{Colour, Context, CreateEmbed, CreateEmbedFooter, CreateMessage};

use crate::{
    guildfile::GuildSettings, helpers::craft_bible_verse_embed, nay, reading_scheudle::Reading,
};

/// uses local timezone
pub fn get_time_until_7am() -> Option<std::time::Duration> {
    let now = Local::now();

    // Calculate the next 7 AM
    let next_7am = (now + chrono::Duration::days(1))
        .with_hour(7)?
        .with_minute(0)?
        .with_second(0)?;

    let Ok(duration) = next_7am.signed_duration_since(now).to_std() else {
        nay!("Failed to get time until next 7am!");
        return None;
    };

    Some(duration)
}

pub async fn spam_daily_verse(
    ctx: &Context,
    verse: &BibleLookup,
    bible: &Bible,
    guilds: &Vec<GuildSettings>,
) {
    for guild in guilds {
        if let Some(channel_id) = guild.get_daily_verse_channel() {
            if let Some(embed) = craft_bible_verse_embed(verse.clone(), &bible) {
                let builder = CreateMessage::new().embed(embed);

                let msg = channel_id.send_message(&ctx.http, builder).await;
                if let Err(e) = msg {
                    nay!("Failed to send daily verse message: {}", e);
                }
            }
        }
    }
}

pub async fn spam_reading_schedule(
    ctx: &Context,
    guilds: &Vec<GuildSettings>,
    reading: Option<Reading>,
    bible: &Bible,
) {
    for guild in guilds {
        if let Some(reading_schedule_channel_id) = guild.get_reading_schedule_channel() {
            // create the embed
            let embed = if let Some(reading) = reading.clone() {
                CreateEmbed::new()
                    .title(format!("📖 Daily Reading"))
                    .description(format!(
                        "Today's reading: {} {} through {} {}",
                        BibleLookup::capitalize_book(&reading.start.book),
                        reading.start.chapter,
                        BibleLookup::capitalize_book(&reading.end.book),
                        reading.end.chapter
                    ))
                    .color(Colour::GOLD)
                    .footer(CreateEmbedFooter::new(format!(
                        "From the {} Bible.",
                        bible.get_translation()
                    )))
            } else {
                CreateEmbed::new()
                .title(format!("📖 Daily Reading"))
                .description(format!("No reading for today! You have completed the Bible this year!\nPlease use this time to catch up or reread missed chapters."))
                .color(Colour::GOLD)
                .footer(CreateEmbedFooter::new(format!("From the {} Bible.", bible.get_translation())))
            };

            let builder = CreateMessage::new().embed(embed);
            let msg = reading_schedule_channel_id
                .send_message(&ctx.http, builder)
                .await;
            if let Err(e) = msg {
                nay!("Failed to send reading schedule message: {}", e);
            }
        }
    }
}
