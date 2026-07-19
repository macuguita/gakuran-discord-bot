use super::apply::Question;
use crate::Context;
use crate::db::tutor_application::{has_pending_application, insert_application};
use anyhow::Result;
use poise::serenity_prelude as serenity;
use std::fmt::Write;
use std::time::Duration;

const QUESTIONS: &[Question] = &[
    Question {
        key: "subjects",
        prompt: "What subjects are you comfortable teaching, and what is the highest level you've completed in each?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "why_teach",
        prompt: "Why do you want to become a teacher for this server?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "when_helped",
        prompt: "Describe a time you helped someone understand a difficult concept. What approach did you use?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "handle_situation",
        prompt: "Imagine you're leading a teaching session and a few students keep talking over you, distracting others who are trying to learn. How would you handle the situation while keeping the session respectful and productive?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "unknown_answer",
        prompt: "How would you respond if you didn't know the answer to a student's question?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "great_teacher_qualities",
        prompt: "What do you think makes a great teacher? List 3-5 qualities and briefly explain why they matter.",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "frustrated_student",
        prompt: "How would you handle a student who becomes frustrated, discouraged, or says, \"I'm just not smart enough for this\"?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "balance_responsibilities",
        prompt: "Teachers in this server are volunteers and are expected to be dependable. How will you balance teaching responsibilities with your life?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "teach_a_concept",
        prompt: "Choose a concept from one of the subjects you want to teach and explain it as if you're teaching it to someone learning it for the first time.",
        min_len: Some(600),
        max_len: Some(800),
    },
    Question {
        key: "available_days",
        prompt: "What days of the week are you generally available?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "available_hours",
        prompt: "How many hours can you be active for your available days?",
        min_len: None,
        max_len: None,
    },
    Question {
        key: "additional_info",
        prompt: "Is there anything else you'd like us to know that would help us understand why you'd be a good teacher in this community?",
        min_len: None,
        max_len: None,
    },
];

#[poise::command(slash_command)]
#[allow(clippy::too_many_lines)]
pub async fn applytutor(ctx: Context<'_>) -> Result<()> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used inside of a server.")
            .await?;
        return Ok(());
    };

    if has_pending_application(&ctx.data().db, guild_id, ctx.author().id).await? {
        ctx.send(
            poise::CreateReply::default()
                .content("You already have a pending application.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let cfg = crate::db::appconfig::get_app_config(&ctx.data().db, guild_id).await?;
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
                        "You took too long to respond. Use `/applytutor` to start again.",
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

    let answers_json = serde_json::to_string(&answers)?;

    let app_id =
        insert_application(&ctx.data().db, guild_id, ctx.author().id, &answers_json).await?;

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
        .description("Full responses attached below.")
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Application #{app_id}"
        )))
        .color(0x58_65_F2);

    let buttons = serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(format!("tutor_app_accept_{app_id}"))
            .label("Accept")
            .style(serenity::ButtonStyle::Success),
        serenity::CreateButton::new(format!("tutor_app_deny_{app_id}"))
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
