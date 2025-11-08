use crate::discord_helpers::command_response;
use crate::guildfile::GuildSettings;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, GuildId,
    Permissions, ResolvedOption, ResolvedValue,
};

pub async fn run(
    options: &[ResolvedOption<'_>],
    ctx: &Context,
    command: &CommandInteraction,
    guild: &GuildId,
) {
    let Some(ResolvedOption {
        value: ResolvedValue::String(option),
        ..
    }) = options.first()
    else {
        // error message
        command_response(
            ctx,
            command,
            "You must specify an option: `daily_verse`, `reading_schedule`, or `remove`",
        )
        .await;
        return;
    };

    let Some(ResolvedOption {
        value: ResolvedValue::Channel(channel),
        ..
    }) = options.get(1)
    else {
        // error message
        command_response(ctx, command, "You must specify a channel").await;
        return;
    };

    let mut guild_file = GuildSettings::get(guild);

    match option.to_lowercase().as_str() {
        "daily_verse" => {
            guild_file.set_daily_verse_channel(channel.id.clone());
            command_response(
                ctx,
                command,
                "Channel is now registered as the daily verse channel!",
            )
            .await;
        }
        "reading_schedule" => {
            guild_file.set_reading_schedule_channel(channel.id.clone());
            command_response(
                ctx,
                command,
                "Channel is now registered as the reading schedule channel",
            )
            .await;
        }
        "remove" => {
            let channel_id = channel.id.get();
            guild_file.clear_channel_by_id(channel_id);
            command_response(
                ctx,
                command,
                "Channel has been removed from the registered channels.",
            )
            .await;
        }
        _ => {
            // error message
            command_response(
                ctx,
                command,
                "You must specify an option: `daily_verse`, `reading_schedule`, or `remove`",
            )
            .await;
        }
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("register_channel")
        .description("Register channels for daily verses or reading schedules")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "action",
                "`daily_verse` (add), `reading_schedule` (add) or `remove` to remove a channel",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Channel,
                "channel",
                "The channel to act on",
            )
            .required(true),
        )
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(false)
}
