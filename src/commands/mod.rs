use crate::{
    helpers::{command_response, craft_bible_verse_embed},
    nay,
};
use bible_lib::{Bible, BibleLookup};
use serenity::all::{
    ChannelId, Colour, CommandInteraction, Context, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
};

pub(crate) mod chapter;
pub(crate) mod random_verse;
pub(crate) mod reading_calc;
pub(crate) mod register_channel;

pub async fn send_bible_verse(
    bible_lookup: BibleLookup,
    ctx: &Context,
    cmd: &CommandInteraction,
    bible: &Bible,
) {
    let Some(embed) = craft_bible_verse_embed(bible_lookup.clone(), bible) else {
        command_response(ctx, cmd, format!("Verse not found: {}", bible_lookup)).await;
        return;
    };

    let msg = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(embed),
            ),
        )
        .await;
    if let Err(e) = msg {
        command_response(ctx, cmd, format!("Failed to send message: {}", e)).await;
        nay!("Failed to send message: {}", e);
    }
}

pub async fn send_bible_chapter<S: Into<String>>(
    book: S,
    chapter: u32,
    ctx: &Context,
    cmd: &CommandInteraction,
    channel: &ChannelId,
    bible: &Bible,
) {
    let book = book.into();
    let book = book.to_lowercase();
    let book = book.as_str();
    let Ok(mut chapter_text) = bible.get_chapter(book, chapter, true) else {
        command_response(
            ctx,
            cmd,
            format!(
                "Chapter not found: {} {} (not found in {} books)",
                book,
                chapter,
                bible.get_books().len()
            ),
        )
        .await;
        return;
    };

    // split the message into multiple messages if it's too long (4096 characters)
    let mut messages = Vec::new();

    while chapter_text.len() > 4096 {
        let split_at = chapter_text.char_indices().nth(4096).unwrap().0;
        let clone = chapter_text.clone();
        let (msg, rest) = clone.split_at(split_at);
        chapter_text = rest.to_string();
        messages.push(msg.to_string());
    }
    messages.push(chapter_text);

    // send the first value in messages
    let title = if messages.len() > 1 {
        format!(
            "📖 {} {} (1/{})",
            BibleLookup::capitalize_book(&book.to_string()),
            chapter,
            messages.len()
        )
    } else {
        format!(
            "📖 {} {}",
            BibleLookup::capitalize_book(&book.to_string()),
            chapter
        )
    };
    let embed = CreateEmbed::new()
        .title(title)
        .description(messages[0].clone())
        .color(Colour::GOLD)
        .footer(CreateEmbedFooter::new(
            "From the King James Version Bible.".to_string(),
        ));

    let msg = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(embed),
            ),
        )
        .await;
    if let Err(e) = &msg {
        command_response(ctx, cmd, format!("Failed to send message: {}", e)).await;
        nay!("Failed to send message: {}", e);
    }

    if messages.len() > 1 {
        for x in 1..messages.len() {
            let chunk = &messages[x];
            let embed = CreateEmbed::new()
                .title(format!(
                    "📖 {} {} ({}/{})",
                    BibleLookup::capitalize_book(&book.to_string()),
                    chapter,
                    x + 1,
                    messages.len()
                ))
                .description(chunk)
                .color(Colour::GOLD)
                .footer(CreateEmbedFooter::new(
                    "From the King James Version Bible.".to_string(),
                ));

            let builder = CreateMessage::new().embed(embed);

            let msg = channel.send_message(&ctx.http, builder).await;
            if let Err(e) = &msg {
                command_response(ctx, cmd, format!("Failed to send message: {}", e)).await;
                nay!("Failed to send message: {}", e);
            }
        }
    }
}
