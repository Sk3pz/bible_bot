use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use bible_lib::{Bible, BibleLookup};
use chrono::Local;
use serenity::{
    all::{
        ActivityData, Context, CreateMessage, EventHandler, GuildId, Interaction, Message,
        OnlineStatus, Ready, ResumedEvent,
    },
    async_trait,
};

use crate::{
    commands,
    daily_handler::{get_time_until_7am, spam_daily_verse, spam_reading_schedule},
    guildfile::GuildSettings,
    helpers::{command_response, craft_bible_verse_embed, register_command},
    hey, nay, reading_scheudle, yay,
};

pub(crate) struct Handler {
    pub bible: Arc<Bible>,
    pub subprocess_running: Arc<AtomicBool>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
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
                    let today = Local::now().date_naive();

                    // get today's reading schedule
                    let reading = reading_scheudle::calculate_reading_for_day(&today, &bible);

                    // get all guilds that have a settings file (guilds that have not configured their daily update channels can be skipped)
                    let guilds = GuildSettings::get_guild_files();

                    // daily verse
                    let daily_verse = bible.random_verse();
                    spam_daily_verse(&ctx, &daily_verse, &bible, &guilds).await;

                    // reading schedule
                    spam_reading_schedule(&ctx, &guilds, reading, &bible).await;

                    // wait until the next 7am
                    let Some(wait_duration) = get_time_until_7am() else {
                        nay!("Failed to get duration until 7am, skipping this iteration!");
                        continue;
                    };
                    tokio::time::sleep(wait_duration).await;
                }
            });
        }
    }

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

        ctx.set_presence(
            Some(ActivityData::custom("Reading the scriptures")),
            OnlineStatus::Online,
        );
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        hey!("Resumed");
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

            // if guild.is_none() {
            //     command_response(&ctx, &command, "This command can only be used in a server").await;
            //     return;
            // }
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
