use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use std::time::Duration;

const QUESTIONS: &[(&str, &str)] = &[
    ("roblox_username", "What is your Roblox username?"),
    ("in_game_name", "What is your in-game character's name?"),
    ("character_backstory", "Tell us a little bit about your character."),
    ("age", "How old are you?"),
    ("uses_vc", "Do you mainly use voice chat or text chat?"),
    ("roleplay_experience", "Tell us about your roleplay experience. Minimum of 2 sentences."),
    ("why_join", "Tell us why you wanna join our community."),
    ("rule", "What is the servers 3rd rule?"),
];

#[poise::command(slash_command)]
pub async fn apply(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used inside of a server.").await?;
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
        .say(ctx, "Let's get started on your application! You have 5 minutes per question.")
        .await?;

    let mut answers: Vec<(String, String)> = Vec::new();

    for (key, prompt) in QUESTIONS {
        dm_channel.say(ctx, *prompt).await?;

        let reply = dm_channel
            .id
            .await_reply(ctx)
            .author_id(ctx.author().id)
            .timeout(Duration::from_mins(5))
            .await;

        if let Some(msg) = reply { answers.push(((*key).to_string(), msg.content.clone())) } else {
            dm_channel
                .say(ctx, "You took too long to respond. Use `/apply` to start again.")
                .await?;
            return Ok(());
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
    let mut embed = serenity::CreateEmbed::default()
        .title(author.name.clone())
        .field("Discord user", format!("<@{}>", author.id), false)
        .footer(serenity::CreateEmbedFooter::new(format!("Application #{app_id}")))
        .color(0x58_65_F2);

    for (key, prompt) in QUESTIONS {
        let value = answers.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()).unwrap_or_default();
        embed = embed.field(*prompt, value, false);
    }

    let buttons = serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(format!("app_accept_{app_id}")).label("Accept").style(serenity::ButtonStyle::Success),
        serenity::CreateButton::new(format!("app_deny_{app_id}")).label("Deny").style(serenity::ButtonStyle::Danger),
    ]);

    channel_id
        .send_message(ctx, serenity::CreateMessage::default().embed(embed).components(vec![buttons]))
        .await?;

    dm_channel
        .say(ctx, "Your application has been submitted! You'll be notified here once it's reviewed.")
        .await?;

    Ok(())
}
