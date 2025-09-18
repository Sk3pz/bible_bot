use bible_lib::Bible;
use serenity::all::{ChannelId, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue};
use crate::command_response;
use crate::commands::send_bible_chapter;

pub async fn run(options: &[ResolvedOption<'_>], ctx: &Context, cmd: &CommandInteraction, channel: &ChannelId, bible: &Bible) {
    if let Some(ResolvedOption { value: ResolvedValue::String(book), .. }) = options.get(0) {
        if let Some(ResolvedOption { value: ResolvedValue::Integer(chapter), .. }) = options.get(1) {
            send_bible_chapter(*book, *chapter as u32, ctx, cmd, channel, bible).await;
        } else {
            command_response(ctx, cmd, "You must specify a chapter!").await;
        }
    } else {
        command_response(ctx, cmd, "You must specify a book of the Bible!").await;
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("chapter")
        .description("Specify a chapter from the bible")
        .add_option(CreateCommandOption::new(CommandOptionType::String, "book", "The book of the Bible")
            .required(true))
        .add_option(CreateCommandOption::new(CommandOptionType::Integer, "chapter", "The chapter of the book")
            .required(true))
        .dm_permission(true)
}