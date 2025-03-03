use serenity::all::{CommandInteraction, Context, CreateCommand};
use crate::bible_data::Bible;
use crate::commands::send_bible_verse;

pub async fn run(ctx: &Context, cmd: &CommandInteraction, bible: &Bible) {
    // get a random verse
    let lookup = bible.random_verse();
    // send the embed
    send_bible_verse(lookup, ctx, cmd, bible).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("random_verse")
        .description("Get a random verse from the Bible")
        .dm_permission(true)
}