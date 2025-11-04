use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use bible_lib::{Bible, BibleLookup};
use chrono::Local;
use serenity::{
    all::{
        ActivityData, Context, CreateMessage, EventHandler, Interaction, Message, OnlineStatus,
        Ready, ResumedEvent,
    },
    async_trait,
};

use crate::{
    commands,
    daily_msg_handler::{get_time_until_7am, spam_daily_verse, spam_reading_schedule},
    daily_verse::DailyVerseHandler,
    guildfile::GuildSettings,
    helpers::{command_response, craft_bible_verse_embed, register_command},
    nay, reading_scheudle, say, yay,
};

pub(crate) struct Handler {
    pub bible: Arc<Bible>,
    pub subprocess_running: Arc<AtomicBool>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from bots
        if msg.author.bot {
            return;
        }

        // detect bible verses
        let verses = BibleLookup::detect_from_string(&msg.content);
        // send the verses
        for verse in verses {
            // create the embed with the bible verse
            if let Some(embed) = craft_bible_verse_embed(verse.clone(), &self.bible) {
                let builder = CreateMessage::new().embed(embed).reference_message(&msg);

                let msg = msg.channel_id.send_message(&ctx.http, builder).await;
                if let Err(e) = msg {
                    nay!("Failed to send message: {}", e);
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        // unregister all commands
        // let commands = Command::get_global_commands(&ctx.http).await.unwrap();
        // for command in commands {
        //     if let Err(e) = Command::delete_global_command(&ctx.http, command.id).await {
        //         nay!("Failed to delete command: {}", e);
        //     }
        // }

        // register commands
        register_command(&ctx, commands::random_verse::register()).await;
        register_command(&ctx, commands::chapter::register()).await;
        register_command(&ctx, commands::register_channel::register()).await;
        register_command(&ctx, commands::reading_calc::register()).await;

        yay!("{} is connected!", ready.user.name);

        ctx.set_presence(
            Some(ActivityData::custom("Reading the scriptures")),
            OnlineStatus::Online,
        );

        // ctx reference
        let ctx = Arc::new(ctx);
        // bible reference
        let bible = Arc::clone(&self.bible);
        // running reference
        //let subprocess_running = Arc::clone(&self.subprocess_running);

        if !self.subprocess_running.load(Ordering::Relaxed) {
            // store that we are running
            self.subprocess_running.store(true, Ordering::Relaxed);

            tokio::spawn(async move {
                // main loop, run until subprocess_running is false
                loop {
                    let today = Local::now().date_naive();

                    // get today's reading schedule
                    let reading = reading_scheudle::calculate_reading_for_day(&today, &bible);

                    // get all guilds that have a settings file (guilds that have not configured their daily update channels can be skipped)
                    let guilds = GuildSettings::get_guild_files();

                    // daily verse
                    let mut verse_handler = DailyVerseHandler::get(&bible);
                    // update the daily verse
                    verse_handler.set_new_verse(&bible);
                    let mut daily_verse = verse_handler.get_verse();
                    spam_daily_verse(&ctx, &daily_verse, &bible, &guilds).await;

                    // refresh in case spam_daily_verse modified it
                    verse_handler.refresh(&bible);
                    daily_verse = verse_handler.get_verse();

                    // set the status to the daily verse
                    ctx.set_presence(
                        Some(ActivityData::custom(format!(
                            "Daily Verse: {}",
                            daily_verse
                        ))),
                        OnlineStatus::Online,
                    );

                    // reading schedule
                    spam_reading_schedule(&ctx, &guilds, reading, &bible).await;

                    // wait until the next 7am
                    let Some(wait_duration) = get_time_until_7am() else {
                        nay!("Failed to get duration until 7am, skipping this iteration!");
                        continue;
                    };
                    let hours_remaining = wait_duration.as_secs() / 3600;
                    let minutes_remaining = (wait_duration.as_secs() % 3600) / 60;
                    let seconds_remaining = wait_duration.as_secs() % 60;
                    say!(
                        "Committed my daily spam. Time until next spam: {}:{}:{}",
                        hours_remaining,
                        minutes_remaining,
                        seconds_remaining
                    );
                    tokio::time::sleep(wait_duration).await;
                }
            });
        }
    }

    async fn resume(&self, ctx: Context, _: ResumedEvent) {
        let verse_handler = DailyVerseHandler::get(&self.bible);

        let daily_verse = verse_handler.get_verse();
        // set the status to the daily verse
        ctx.set_presence(
            Some(ActivityData::custom(format!(
                "Daily Verse: {}",
                daily_verse
            ))),
            OnlineStatus::Online,
        );
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let command_name = command.data.name.as_str();
                //let sender = &command.user;
                let guild = command.guild_id;
                //let guild_id = guild.unwrap();
                let command_options = &command.data.options();
                let channel = command.channel_id;

                match command_name {
                    "random_verse" => {
                        commands::random_verse::run(&ctx, &command, &self.bible).await;
                    }
                    "chapter" => {
                        commands::chapter::run(
                            command_options,
                            &ctx,
                            &command,
                            &channel,
                            &self.bible,
                        )
                        .await;
                    }
                    "reading_calc" => {
                        commands::reading_calc::run(command_options, &ctx, &command, &self.bible)
                            .await;
                    }
                    "register_channel" => {
                        commands::register_channel::run(
                            command_options,
                            &ctx,
                            &command,
                            &guild.unwrap(),
                        )
                        .await;
                    }
                    _ => {
                        command_response(&ctx, &command, "Unknown command!").await;
                    }
                }
            }
            _ => {}
        }
    }
}
