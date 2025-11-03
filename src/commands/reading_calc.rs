use bible_lib::{Bible, BibleLookup};
use chrono::{Datelike, NaiveDate};
use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
    ResolvedOption, ResolvedValue,
};

use crate::{helpers::command_response, nay, reading_scheudle::calculate_reading_for_day};

pub async fn run(
    options: &[ResolvedOption<'_>],
    ctx: &Context,
    cmd: &CommandInteraction,
    bible: &Bible,
) {
    let date = if let Some(ResolvedOption {
        // MONTH
        value: ResolvedValue::Integer(month),
        ..
    }) = options.get(0)
    {
        if let Some(ResolvedOption {
            // DAY
            value: ResolvedValue::Integer(day),
            ..
        }) = options.get(1)
        {
            if let Some(ResolvedOption {
                // YEAR
                value: ResolvedValue::Integer(year),
                ..
            }) = options.get(2)
            {
                (month.clone(), day.clone(), year.clone() as i32)
            } else {
                // NO YEAR SPECIFIED
                // current year
                let current_year = chrono::Utc::now().year();
                (month.clone(), day.clone(), current_year)
            }
        } else {
            command_response(ctx, cmd, "You must specify a day!").await;
            return;
        }
    } else {
        command_response(ctx, cmd, "You must specify a month!").await;
        return;
    };

    let year = date.2;
    let month = date.0 as u32;
    let day = date.1 as u32;

    let date = NaiveDate::from_ymd_opt(year, month, day);

    match date {
        Some(date) => {
            let reading = calculate_reading_for_day(&date, bible);
            let embed = if let Some(reading) = reading.clone() {
                CreateEmbed::new()
                    .title(format!("📖 Daily Reading for {}", date.format("%B %d, %Y")))
                    .description(format!(
                        "Today's reading: {} {} through {} {}",
                        BibleLookup::capitalize_book(&reading.start.book),
                        reading.start.chapter,
                        BibleLookup::capitalize_book(&reading.end.book),
                        reading.end.chapter
                    ))
                    .color(Colour::GOLD)
                    .footer(CreateEmbedFooter::new(format!(
                        "From the {} Bible.",
                        bible.get_translation()
                    )))
            } else {
                CreateEmbed::new()
                                .title(format!("📖 Daily Reading for {}", date.format("%B %d, %Y")))
                                .description(format!("No reading for today! You have completed the Bible this year!\nPlease use this time to catch up or reread missed chapters."))
                                .color(Colour::GOLD)
                                .footer(CreateEmbedFooter::new(format!("From the {} Bible.", bible.get_translation())))
            };

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
        }
        None => {
            command_response(ctx, cmd, "Invalid date provided.").await;
        }
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("reading_calc")
        .description("Check what the daily reading will be for a specific date")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "month",
                "The month (numeric: 1-12)",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::Integer, "day", "The day (numeric)")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::Integer, "year", "The year (optional)")
                .required(false),
        )
        .dm_permission(true)
}
