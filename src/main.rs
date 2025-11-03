use bible_lib::{Bible, Translation};
use serenity::all::GatewayIntents;
use serenity::Client;
use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::discord_handler::Handler;

pub mod guildfile;
pub mod helpers;
pub mod logging;
pub mod reading_scheudle;

mod commands;
mod discord_handler;

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
