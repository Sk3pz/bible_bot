use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use bible_lib::{Bible, BibleLookup, Translation};
use chrono::{Duration, NaiveTime};
use chrono_tz::America;
use serenity::all::{ActivityData, Colour, Command, CommandInteraction, CreateCommand, CreateEmbed, CreateEmbedFooter, CreateMessage, GatewayIntents, GuildId, Interaction, Message, OnlineStatus, Ready, ResumedEvent};
use serenity::{async_trait, Client};
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::client::{Context, EventHandler};

use crate::guildfile::{GuildSettings};

pub mod logging;
pub mod guildfile;
mod reading_scheudle;

mod commands;

// fn detect_bible_verses(text: &str) -> Vec<BibleLookup> {
//     let mut verses = Vec::new();

//     let text = text.to_lowercase();

//     //let regex = regex::Regex::new(r"\b(?:genesis|exodus|leviticus|numbers|deuteronomy|joshua|judges|ruth|1\s?samuel|2\s?samuel|1\s?kings|2\s?kings|1\s?chronicles|2\s?chronicles|ezra|nehemiah|esther|job|psalms|proverbs|ecclesiastes|song\sof\ssolomon|isaiah|jeremiah|lamentations|ezekiel|daniel|hosea|joel|amos|obadiah|jonah|micah|nahum|habakkuk|zephaniah|haggai|zechariah|malachi|matthew|mark|luke|john|acts|romans|1\s?corinthians|2\s?corinthians|galatians|ephesians|philippians|colossians|1\s?thessalonians|2\s?thessalonians|1\s?timothy|2\s?timothy|titus|philemon|hebrews|james|1\s?peter|2\s?peter|1\s?john|2\s?john|3\s?john|jude|revelation)\s+\d+:\d+\b").unwrap();
//     let regex = regex::Regex::new(r"\b(?:genesis|exodus|leviticus|numbers|deuteronomy|joshua|judges|ruth|1\s?samuel|2\s?samuel|1\s?kings|2\s?kings|1\s?chronicles|2\s?chronicles|ezra|nehemiah|esther|job|psalms|proverbs|ecclesiastes|song\sof\ssolomon|isaiah|jeremiah|lamentations|ezekiel|daniel|hosea|joel|amos|obadiah|jonah|micah|nahum|habakkuk|zephaniah|haggai|zechariah|malachi|matthew|mark|luke|john|acts|romans|1\s?corinthians|2\s?corinthians|galatians|ephesians|philippians|colossians|1\s?thessalonians|2\s?thessalonians|1\s?timothy|2\s?timothy|titus|philemon|hebrews|james|1\s?peter|2\s?peter|1\s?john|2\s?john|3\s?john|jude|revelation)\s+\d+:\d+(?:-\d+)?\b").unwrap();

//     for instance in regex.find_iter(&text) {
//         let instance = instance.as_str();
//         // to handle cases like `1 samuel` and `Song of Solomon`, split by ':' first and then split by whitespace
//         let mut parts = instance.split(':');
//         // split the first part by whitespace
//         let book_chapter = parts.next().unwrap().split_whitespace();
//         let count = book_chapter.clone().count();
//         let chapter = book_chapter.clone().last().unwrap().parse::<u32>().unwrap();
//         let book = book_chapter.take(count - 1).collect::<Vec<&str>>().join(" ").to_lowercase();

//         // handle cases where the verse is a range (i.e. `1-3`)
//         let verse_part = parts.next().unwrap();
//         if verse_part.contains('-') {
//             let verse_split = verse_part.split('-');
//             let verse = verse_split.clone().next().unwrap().parse::<u32>().unwrap();
//             let thru_verse = verse_split.clone().last().unwrap().parse::<u32>().unwrap();
//             verses.push(BibleLookup {
//                 book,
//                 chapter,
//                 verse,
//                 thru_verse: Some(thru_verse),
//             });
//         } else {
//             let verse = verse_part.parse::<u32>().unwrap();
//             verses.push(BibleLookup {
//                 book,
//                 chapter,
//                 verse,
//                 thru_verse: None,
//             });
//         }
//     }

//     verses
// }

pub fn craft_bible_verse_embed(verse: BibleLookup, bible: &Bible) -> Option<CreateEmbed> {
    if let Ok(max_verse) = bible.get_max_verse(&verse.book, verse.chapter) {
        if verse.verse > max_verse {
            return Some(CreateEmbed::new()
                .title(format!("📖 {}", verse))
                .description(format!("That verse does not exist! {} {} only has {} verses.",
                                     BibleLookup::capitalize_book(&verse.book), verse.chapter, max_verse))
                .color(Colour::GOLD));
        }
    }

    if let Ok(verse_text) = bible.get_verse(verse.clone(), true) {
        if verse_text.len() > 2048 {
            return Some(CreateEmbed::new()
                    .title(format!("📖 {}", verse))
                    .description("I am sorry but that would be too long for a message!")
                    .color(Colour::GOLD)
                    .footer(CreateEmbedFooter::new("Tip: use `/chapter <book> <chapter>` to show a full chapter".to_string())));
        }

        if let Some(thru_verse) = verse.thru_verse.clone() {
            if thru_verse > bible.get_max_verse(&verse.book, verse.chapter).unwrap_or(0) {
                return Some(CreateEmbed::new()
                        .title(format!("📖 {}", verse))
                    .description(format!("That verse range extends past the amount of verses! {} {} only has {} verses.",
                                         BibleLookup::capitalize_book(&verse.book), verse.chapter, bible.get_max_verse(&verse.book, verse.chapter).unwrap_or(0)))
                        .color(Colour::GOLD));
            }
        }

        Some(CreateEmbed::new()
            .title(format!("📖 {}", verse))
            .description(verse_text)
            .color(Colour::GOLD)
            .footer(CreateEmbedFooter::new(format!("From the {} Bible.", bible.get_translation()))))
    } else {
        None
    }
}

pub async fn command_response<S: Into<String>>(ctx: &Context, command: &CommandInteraction, msg: S) {
    let data = CreateInteractionResponseMessage::new().content(msg.into());
    let builder = CreateInteractionResponse::Message(data);
    if let Err(err) = command.create_response(&ctx.http, builder).await {
        nay!("Failed to respond to command: {}", err)
    }
}

pub async fn register_command(ctx: &Context, cmd: CreateCommand) {
    if let Err(e) = Command::create_global_command(&ctx.http, cmd).await {
        nay!("Failed to register a command: {}", e);
    }
}

struct Handler {
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
                let builder = CreateMessage::new()
                    .embed(embed)
                    .reference_message(&msg);

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

        yay!("{} is connected!", ready.user.name);

        // // create the role selection menu
        // 
        // // get the channel "role-selection" with id 1236744668688154705
        // let role_selection_channel = ctx.http.get_channel(
        //     ChannelId::new(1236744668688154705_u64))
        //     .await.unwrap();
        // 
        // // create the role selection menu embed
        // let embed = CreateEmbed::new()
        //     .title("Role Selection")
        //     .description("Select roles based on your beliefs.")
        //     .fields(vec![
        //         ("✝️Christian (Protestant)", "Align with the Protestant branch (any denomination, including non-denominational)", false),
        //         ("✝️Christian (Catholic)", "Member of the Catholic church or align with their views.", false),
        //         ("✝️Christian (Orthodox)", "Member of the Orthodox church or align with their views.", false),
        //         ("Christian (Other)", "Follows a separate branch of Christianity not listed", false),
        // 
        //         ("✡️Judaism", "Follows the Jewish faith", false),
        //         ("☪️Islam", "Follows the Islamic faith", false),
        //         ("🕉️Buddhism", "Follows the Buddhist faith", false),
        //         ("🕉️Hinduism", "Follows the Hindu faith", false),
        // 
        //         ("Atheist", "Does not believe in God", false),
        //         ("❔Agnostic", "Unsure of the existence of God", false),
        //         ("Other", "Follows a faith not listed", false),
        //     ])
        //     .color(Colour::DARK_TEAL);
        // 
        // let menu = CreateSelectMenu::new("role_selection", CreateSelectMenuKind::String {
        //     options: vec![
        //         CreateSelectMenuOption::new("✝️Christian (Protestant)", "christian_protestant"),
        //         CreateSelectMenuOption::new("✝️Christian (Catholic)", "christian_catholic"),
        //         CreateSelectMenuOption::new("✝️Christian (Orthodox)", "christian_orthodox"),
        //         CreateSelectMenuOption::new("Christian (Other)", "christian_other"),
        // 
        //         CreateSelectMenuOption::new("✡️Judaism", "judaism"),
        //         CreateSelectMenuOption::new("☪️Islam", "islam"),
        //         CreateSelectMenuOption::new("🕉️Buddhism", "buddhism"),
        //         CreateSelectMenuOption::new("🕉️Hinduism", "hinduism"),
        // 
        //         CreateSelectMenuOption::new("Atheist", "atheist"),
        //         CreateSelectMenuOption::new("❔Agnostic", "agnostic"),
        //         CreateSelectMenuOption::new("Other", "other"),
        //     ]
        // })
        //     .max_values(2)
        //     .min_values(1);
        // 
        // let builder = CreateMessage::new()
        //     .content("Select your role based on your beliefs. You can change this later")
        //     .embed(embed)
        //     .select_menu(menu);
        // 
        // let msg = role_selection_channel.id().send_message(&ctx.http, builder).await;
        // if let Err(e) = &msg {
        //     nay!("Failed to send role selection message: {}", e);
        // }

        ctx.set_presence(Some(ActivityData::custom("Reading the scriptures")), OnlineStatus::Online);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        hey!("Resumed");
    }

    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        // spawn daily verse and reading thread
        
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
                    // get the current time, and wait until 7am CST

                    // get current time in CST
                    let now = chrono::Utc::now().with_timezone(&America::Chicago);

                    // calculate the duration until 7am CST for today
                    let Some(seven_am) = NaiveTime::from_hms_opt(7, 0, 0) else {
                        nay!("Failed to create 7am time for scheduling. Skipping this process iteration.");
                        return;
                    };
                    let next_run = now.with_time(seven_am).unwrap();

                    // if it's already past 7am, schedule for tomorrow
                    let next_run = if now < next_run {
                        next_run
                    } else {
                        next_run + Duration::days(1)
                    };

                    let wait_duration = next_run - now;
                    println!("Next run in {:?}", wait_duration);

                    // wait until the next 7am CST
                    tokio::time::sleep(wait_duration.to_std().unwrap()).await;

                    // send the daily verse and reading to all guilds that have a channel named "daily-verse" and "reading-schedule"

                    // get the daily verse
                    let daily_verse = bible.random_verse();
                    // get today's reading
                    let today = chrono::Utc::now().with_timezone(&America::Chicago).date_naive();
                    let reading = reading_scheudle::calculate_reading_for_day(&today, Arc::clone(&bible));
                    
                    // get all guilds
                    for guild_id in &guilds {
                        // get the guild
                        let guild = match guild_id.to_partial_guild(&ctx.http).await {
                            Ok(guild) => guild,
                            Err(e) => {
                                nay!("Failed to get guild {}: {}", guild_id, e);
                                continue;
                            }
                        };

                        // get the guild's file
                        let mut guild_file = GuildSettings::get(&guild.id);

                        // get the daily verse channel and send the verse
                        if let Some(daily_verse_channel_id) = guild_file.get_daily_verse_channel() {
                            if let Some(embed) = craft_bible_verse_embed(daily_verse.clone(), &bible) {
                                let builder = CreateMessage::new()
                                    .embed(embed);

                                let msg = daily_verse_channel_id.send_message(&ctx.http, builder).await;
                                if let Err(e) = msg {
                                    nay!("Failed to send daily verse message: {}", e);
                                }
                            }
                        }

                        // get the reading schedule channel
                        if let Some(reading_schedule_channel_id) = guild_file.get_reading_schedule_channel() {
                            // create the embed
                            let embed = if let Some(reading) = reading.clone() {
                                CreateEmbed::new()
                                .title(format!("📖 Daily Reading"))
                                .description(format!("Today's reading: {} {} through {} {}", 
                                BibleLookup::capitalize_book(&reading.start.book), reading.start.chapter, 
                                BibleLookup::capitalize_book(&reading.end.book), reading.end.chapter))
                                .color(Colour::GOLD)
                                .footer(CreateEmbedFooter::new(format!("From the {} Bible.", bible.get_translation())))
                            } else {
                                CreateEmbed::new()
                                .title(format!("📖 Daily Reading"))
                                .description(format!("No reading for today! You have completed the Bible this year!\nPlease use this time to catch up or reread missed chapters."))
                                .color(Colour::GOLD)
                                .footer(CreateEmbedFooter::new(format!("From the {} Bible.", bible.get_translation())))
                            };

                            let builder = CreateMessage::new()
                                .embed(embed);
                            let msg = reading_schedule_channel_id.send_message(&ctx.http, builder).await;
                            if let Err(e) = msg {
                                nay!("Failed to send reading schedule message: {}", e);
                            }
                        }
                    }
                }
            });
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let command_name = command.data.name.as_str();
                //let sender = &command.user;
                let guild = command.guild_id;
                if guild.is_none() {
                    command_response(&ctx, &command, "This command can only be used in a server").await;
                    return;
                }
                //let guild_id = guild.unwrap();
                let command_options = &command.data.options();
                let channel = command.channel_id;

                match command_name {
                    "random_verse" => {
                        commands::random_verse::run(&ctx, &command, &self.bible).await;
                    }
                    "chapter" => {
                        commands::chapter::run(command_options, &ctx, &command, &channel, &self.bible).await;
                    }
                    "register_channel" => {
                        commands::register_channel::run(command_options, &ctx, &command, &guild.unwrap()).await;
                    }
                    _ => {
                        command_response(&ctx, &command, "Unknown command!").await;
                    }
                }
            }
            // Interaction::Component(component) => {
            //     if component.data.custom_id.as_str() == "role_selection" {
            //         if let ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
            //             let core_roles = vec![
            //                 RoleId::new(1236753755232276540_u64), // christian_protestant
            //                 RoleId::new(1236753890540519544_u64), // christian_catholic
            //                 RoleId::new(1236753932559057026_u64), // christian_orthodox
            //                 RoleId::new(1236754177900675163_u64), // christian_other
            // 
            //                 RoleId::new(1236754048888213604_u64), // judaism
            //                 RoleId::new(1236754118614188112_u64), // islam
            //                 RoleId::new(1236754242501349547_u64), // buddhism
            //                 RoleId::new(1236754366732701767_u64), // hinduism
            // 
            //                 RoleId::new(1236754578490265651_u64), // atheist
            //                 RoleId::new(1236754645334884484_u64), // agnostic
            //                 RoleId::new(1236754772950777898_u64), // other
            // 
            //                 RoleId::new(1236773068102438924_u64), // christian
            //             ];
            // 
            //             let mut roles = vec![];
            //             for value in values {
            //                 match value.as_str() {
            //                     "christian_protestant" => {
            //                         roles.push(core_roles[0]);
            //                         roles.push(core_roles[11]);
            //                     },
            //                     "christian_catholic" => {
            //                         roles.push(core_roles[1]);
            //                         roles.push(core_roles[11]);
            //                     },
            //                     "christian_orthodox" => {
            //                         roles.push(core_roles[2]);
            //                         roles.push(core_roles[11]);
            //                     },
            //                     "christian_other" => {
            //                         roles.push(core_roles[3]);
            //                         roles.push(core_roles[11]);
            //                     },
            // 
            //                     "judaism" => roles.push(core_roles[4]),
            //                     "islam" => roles.push(core_roles[5]),
            //                     "buddhism" => roles.push(core_roles[6]),
            //                     "hinduism" => roles.push(core_roles[7]),
            // 
            //                     "atheist" => roles.push(core_roles[8]),
            //                     "agnostic" => roles.push(core_roles[9]),
            //                     "other" => roles.push(core_roles[10]),
            //                     _ => {}
            //                 }
            //             }
            //             let Some(member) = &component.member else {
            //                 nay!("Failed to get member");
            //                 return;
            //             };
            // 
            //             // update the component to show the roles have been added
            //             if let Err(e) = component.create_response(&ctx.http, CreateInteractionResponse::Acknowledge).await {
            //                 nay!("Failed to update component: {}", e);
            //             };
            // 
            //             // remove roles
            //             if let Err(e) = member.remove_roles(&ctx.http, core_roles.as_slice()).await {
            //                 nay!("Failed to remove roles: {}", e);
            //             }
            // 
            //             // add the new roles
            //             if let Err(e) = member.add_roles(&ctx.http, roles.as_slice()).await {
            //                 nay!("Failed to add roles: {}", e);
            //             }
            //         } else {
            //             nay!("Invalid select menu kind: {:?}", component.data.kind);
            //         }
            //     }
            // }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    yay!("✝ Bible Bot is starting up!");

    dotenv::dotenv().expect("Failed to load .env file");

    let Ok(token) = env::var("DISCORD_TOKEN") else {
        nay!("DISCORD_TOKEN not found in environment");
        return;
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    say!("Loading bible into ram...");
    //let bible = Bible::parse("bible_translations/kjv.txt");
    let Ok(bible) = Bible::new(Translation::AmericanKingJames) else {
        nay!("Failed to load bible");
        return;
    };
    say!("Bible loaded!");

    let Ok(mut client) = Client::builder(token, intents)
        .event_handler(Handler { 
            bible: Arc::new(bible),
            subprocess_running: Arc::new(AtomicBool::new(false)),
         })
        .await
        else {
            nay!("Error creating client");
            return;
        };

    if let Err(err) = client.start().await {
        nay!("Client error: {}", err);
    }
}
