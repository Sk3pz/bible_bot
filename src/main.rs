use bible_lib::{Bible, Translation};
use serenity::all::GatewayIntents;
use serenity::Client;
use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::discord_handler::Handler;

pub mod daily_messages;
pub mod discord_helpers;
pub mod guildfile;
pub mod logging;

mod commands;
mod discord_handler;

const USED_TRANSLATION: Translation = Translation::AmericanStandard;

#[tokio::main]
async fn main() {
    yay!("✝ Bible Bot is starting up!");

    // create the guilds and data directory if it doesn't exist
    let Ok(exists) = std::fs::exists("./guilds") else {
        nay!("Failed to check if guilds directory exists");
        return;
    };
    if !exists {
        if let Err(e) = std::fs::create_dir_all("./guilds") {
            nay!("Failed to create guilds directory: {}", e);
            return;
        };
    }
    let Ok(exists) = std::fs::exists("./data") else {
        nay!("Failed to check if guilds directory exists");
        return;
    };
    if !exists {
        if let Err(e) = std::fs::create_dir_all("./data") {
            nay!("Failed to create data directory: {}", e);
            return;
        };
    }

    // get the env variables
    dotenv::dotenv().expect("Failed to load .env file");

    let Ok(token) = env::var("DISCORD_TOKEN") else {
        nay!("DISCORD_TOKEN not found in environment");
        return;
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    say!("Loading bible into ram...");
    let Ok(bible) = Bible::new(USED_TRANSLATION) else {
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
