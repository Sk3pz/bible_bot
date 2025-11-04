use bible_lib::{Bible, BibleLookup};
use chrono::{Local, Timelike};
use serenity::all::{Colour, Context, CreateEmbed, CreateEmbedFooter, CreateMessage, GetMessages};

use crate::{
    daily_verse::DailyVerseHandler, guildfile::GuildSettings, nay, reading_scheudle::Reading,
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
    let mut verse = verse.clone();

    // check for sent verse in each guild and update it if necessary
    // this has to be a separate loop because any server no matter the order
    // could have the verse already sent.
    // prevents bugs where some servers get different verses if they deleted the message and the bot restarts
    let found_sent = Vec::new();
    for guild in guilds {
        if let Some(channel_id) = guild.get_daily_verse_channel() {
            // check if the channel already has the verse sent today
            let history_builder = GetMessages::new().limit(20);
            let messages = channel_id.messages(&ctx.http, history_builder).await;
            // check if there is already a daily verse message sent today (for bot restarts)
            if let Ok(msgs) = messages {
                let now = chrono::Utc::now().date_naive();

                if let Some(embed) = msgs
                    .iter()
                    .filter(|m| m.author.bot && m.timestamp.date_naive() == now)
                    .filter_map(|m| m.embeds.first())
                    .find(|embed| {
                        embed
                            .footer
                            .as_ref()
                            .map(|f| f.text.contains("Daily verse"))
                            .unwrap_or(false)
                    })
                {
                    if let Some(title) = &embed.title {
                        let old_verse = title.replace("📖 ", "");
                        let mut handler = DailyVerseHandler::get(bible);
                        let lookup = BibleLookup::detect_from_string(old_verse);
                        if let Some(found_verse) = lookup.first() {
                            // hey!("updating verse: {}", found_verse);
                            verse = found_verse.clone();
                            handler.set_custom_verse(found_verse.clone(), bible);
                        } else {
                            nay!("Failed to detect verse from old daily verse message.");
                            continue;
                        }
                    }
                    // hey!(
                    //     "Found daily verse message for {}, not sending another message.",
                    //     guild.id.clone()
                    // );
                    found_sent.push(channel_id);
                }
            }
        }
    }

    // spam the messages
    for guild in guilds {
        if let Some(channel_id) = guild.get_daily_verse_channel() {
            if found_sent.contains(&channel_id) {
                continue; // already sent in this channel
            }
            let Ok(verse_text) = bible.get_verse(verse.clone(), true) else {
                nay!("Failed to get verse text for daily verse spam.");
                return;
            };

            // create the embed and send it
            let embed = CreateEmbed::new()
                .title(format!("📖 {}", verse))
                .description(verse_text.clone())
                .color(Colour::GOLD)
                .footer(CreateEmbedFooter::new(format!(
                    "Daily verse from the {} Bible.",
                    bible.get_translation()
                )));
            let builder = CreateMessage::new().embed(embed);

            let msg = channel_id.send_message(&ctx.http, builder).await;
            if let Err(e) = msg {
                nay!("Failed to send daily verse message: {}", e);
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
            // ensure today's reading hasn't already been sent (for bot restarts)
            let history_builder = GetMessages::new().limit(20);
            let messages = reading_schedule_channel_id
                .messages(&ctx.http, history_builder)
                .await;
            if let Ok(msgs) = messages {
                let now = chrono::Utc::now().date_naive();

                if let Some(_) = msgs
                    .iter()
                    .filter(|m| m.author.bot && m.timestamp.date_naive() == now)
                    .filter_map(|m| m.embeds.first())
                    .find(|embed| {
                        embed
                            .title
                            .as_ref()
                            .unwrap_or(&String::new())
                            .contains("📖 Daily Reading")
                    })
                {
                    // hey!(
                    //     "Found reading schedule message for {}, not sending another message.",
                    //     guild.id.clone()
                    // );
                    continue; // no need to send the verse again
                }
            }

            // create the embed
            let embed = if let Some(reading) = reading.clone() {
                let description = if reading.start.book == reading.end.book {
                    format!(
                        "Today's Reading: {} {} through {}",
                        BibleLookup::capitalize_book(&reading.start.book),
                        reading.start.chapter,
                        reading.end.chapter
                    )
                } else {
                    format!(
                        "Today's Reading: {} {} through {} {}",
                        BibleLookup::capitalize_book(&reading.start.book),
                        reading.start.chapter,
                        BibleLookup::capitalize_book(&reading.end.book),
                        reading.end.chapter
                    )
                };

                CreateEmbed::new()
                    .title(format!("📖 Daily Reading"))
                    .description(description)
                    .color(Colour::GOLD)
                    .footer(CreateEmbedFooter::new("Come read with us!"))
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
