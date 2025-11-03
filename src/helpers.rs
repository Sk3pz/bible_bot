use bible_lib::{Bible, BibleLookup};
use serenity::all::{
    Colour, Command, CommandInteraction, CreateCommand, CreateEmbed, CreateEmbedFooter,
};
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::client::Context;

use crate::nay;

pub fn craft_bible_verse_embed(verse: BibleLookup, bible: &Bible) -> Option<CreateEmbed> {
    if let Ok(max_verse) = bible.get_max_verse(&verse.book, verse.chapter) {
        if verse.verse > max_verse {
            return Some(
                CreateEmbed::new()
                    .title(format!("📖 {}", verse))
                    .description(format!(
                        "That verse does not exist! {} {} only has {} verses.",
                        BibleLookup::capitalize_book(&verse.book),
                        verse.chapter,
                        max_verse
                    ))
                    .color(Colour::GOLD),
            );
        }
    }

    if let Ok(verse_text) = bible.get_verse(verse.clone(), true) {
        if verse_text.len() > 2048 {
            return Some(
                CreateEmbed::new()
                    .title(format!("📖 {}", verse))
                    .description("I am sorry but that would be too long for a message!")
                    .color(Colour::GOLD)
                    .footer(CreateEmbedFooter::new(
                        "Tip: use `/chapter <book> <chapter>` to show a full chapter".to_string(),
                    )),
            );
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

        Some(
            CreateEmbed::new()
                .title(format!("📖 {}", verse))
                .description(verse_text)
                .color(Colour::GOLD)
                .footer(CreateEmbedFooter::new(format!(
                    "From the {} Bible.",
                    bible.get_translation()
                ))),
        )
    } else {
        None
    }
}

pub async fn command_response<S: Into<String>>(
    ctx: &Context,
    command: &CommandInteraction,
    msg: S,
) {
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
