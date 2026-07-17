use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use std::time::Duration;
use std::fmt::Write;

struct Question {
    key: &'static str,
    prompt: &'static str,
    min_len: Option<usize>,
    max_len: Option<usize>,
}

const QUESTIONS: &[Question] = &[
    Question {
        key: "roblox_username",
        prompt: "What is your Roblox username?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "in_game_name",
        prompt: "What is your in-game character's name?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "character_backstory",
        prompt: "Tell us a little bit about your character.",
        min_len: Some(185),
        max_len: None,
    },
    Question {
        key: "age",
        prompt: "How old are you?",
        min_len: None,
        max_len: Some(2),
    },
    Question {
        key: "uses_vc",
        prompt: "Do you mainly use voice chat or text chat?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "roleplay_experience",
        prompt: "Tell us about your roleplay experience. Minimum of 2 sentences.",
        min_len: Some(200),
        max_len: None,
    },
    Question {
        key: "why_join",
        prompt: "Tell us why you wanna join our community.",
        min_len: Some(150),
        max_len: None,
    },
    Question {
        key: "rule",
        prompt: "What is the servers 3rd rule?",
        min_len: None,
        max_len: None,
    },
];

#[poise::command(slash_command)]
#[allow(clippy::too_many_lines)]
pub async fn apply(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used inside of a server.")
            .await?;
        return Ok(());
    };

    if crate::db::has_pending_application(&ctx.data().db, guild_id, ctx.author().id).await? {
        ctx.send(
            poise::CreateReply::default()
                .content("You already have a pending application.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let cfg = crate::db::get_app_config(&ctx.data().db, guild_id).await?;
    let Some(channel_id) = cfg.as_ref().and_then(|c| c.response_channel) else {
        ctx.send(
            poise::CreateReply::default()
                .content("Applications have not been set up for this server yet.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    let Ok(dm_channel) = ctx.author().create_dm_channel(ctx).await else {
        ctx.send(
            poise::CreateReply::default()
                .content("I couldn't DM you — please enable DMs from server members and try again.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    ctx.send(
        poise::CreateReply::default()
            .content("Check your DMs — I've sent you the application questions!")
            .ephemeral(true),
    )
    .await?;

    dm_channel
        .say(
            ctx,
            "Let's get started on your application! You have 5 minutes per question.",
        )
        .await?;

    let mut answers: Vec<(String, String)> = Vec::new();

    for q in QUESTIONS {
        let prompt = match (q.min_len, q.max_len) {
            (Some(min), Some(max)) => format!("{} (between {min} and {max} characters)", q.prompt),
            (Some(min), None) => format!("{} (minimum {min} characters)", q.prompt),
            (None, Some(max)) => format!("{} (maximum {max} characters)", q.prompt),
            (None, None) => q.prompt.to_string(),
        };

        dm_channel.say(ctx, prompt).await?;

        loop {
            let reply = dm_channel
                .id
                .await_reply(ctx)
                .author_id(ctx.author().id)
                .timeout(Duration::from_mins(5))
                .await;

            let Some(msg) = reply else {
                dm_channel
                    .say(
                        ctx,
                        "You took too long to respond. Use `/apply` to start again.",
                    )
                    .await?;
                return Ok(());
            };

            let len = msg.content.chars().count();

            if let Some(min) = q.min_len
                && len < min
            {
                dm_channel
                    .say(ctx, format!("That answer needs to be at least {min} characters (yours was {len}). Please try again."))
                    .await?;
                continue;
            }

            if let Some(max) = q.max_len
                && len > max
            {
                dm_channel
                    .say(ctx, format!("That answer needs to be at most {max} characters (yours was {len}). Please try again."))
                    .await?;
                continue;
            }

            answers.push((q.key.to_string(), msg.content.clone()));
            break;
        }
    }

    let in_game_name = answers
        .iter()
        .find(|(k, _)| k == "in_game_name")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let answers_json = serde_json::to_string(&answers)?;

    let app_id = crate::db::insert_application(
        &ctx.data().db,
        guild_id,
        ctx.author().id,
        &in_game_name,
        &answers_json,
    )
    .await?;

    let author = ctx.author();

    let mut transcript = format!("Application from {} ({})\n\n", author.name, author.id);
    for q in QUESTIONS {
        let value = answers
            .iter()
            .find(|(k, _)| k == q.key)
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let _ = write!(transcript, "**{}**\n{}\n\n", q.prompt, value);
    }

    let attachment = serenity::CreateAttachment::bytes(
        transcript.into_bytes(),
        format!("application_{app_id}.md"),
    );

    let embed = serenity::CreateEmbed::default()
        .title(format!("Application from {}", author.name))
        .field("Discord user", format!("<@{}>", author.id), true)
        .field("In-game name", &in_game_name, true)
        .description("Full responses attached below.")
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Application #{app_id}"
        )))
        .color(0x58_65_F2);

    let buttons = serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(format!("app_accept_{app_id}"))
            .label("Accept")
            .style(serenity::ButtonStyle::Success),
        serenity::CreateButton::new(format!("app_deny_{app_id}"))
            .label("Deny")
            .style(serenity::ButtonStyle::Danger),
    ]);

    channel_id
        .send_message(
            ctx,
            serenity::CreateMessage::default()
                .embed(embed)
                .components(vec![buttons])
                .add_file(attachment),
        )
        .await?;

    dm_channel
        .say(
            ctx,
            "Your application has been submitted! You'll be notified here once it's reviewed.",
        )
        .await?;

    Ok(())
}
