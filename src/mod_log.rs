use poise::serenity_prelude as serenity;

#[allow(clippy::unreadable_literal)]
pub async fn handle_message_delete(
    ctx: &serenity::Context,
    data: &crate::Data,
    channel_id: serenity::ChannelId,
    deleted_message_id: serenity::MessageId,
    guild_id: Option<serenity::GuildId>,
) -> Result<(), crate::Error> {
    let Some(guild_id) = guild_id else {
        return Ok(());
    };

    let log_channel = crate::db::get_app_config(&data.db, guild_id)
        .await?
        .and_then(|c| c.mod_log_channel);
    let Some(log_channel) = log_channel else {
        return Ok(());
    }; // not configured for this guild

    let cached = ctx
        .cache
        .message(channel_id, deleted_message_id)
        .map(|m| (m.author.tag(), m.content.clone()));

    let embed = match cached {
        Some((author, content)) => serenity::CreateEmbed::new()
            .title("Message Deleted")
            .color(0xED4245)
            .field("Author", author, true)
            .field("Channel", format!("<#{channel_id}>"), true)
            .field(
                "Content",
                if content.is_empty() {
                    "*(no text content)*".into()
                } else {
                    content
                },
                false,
            )
            .timestamp(serenity::Timestamp::now()),
        None => serenity::CreateEmbed::new()
            .title("Message Deleted")
            .color(0xED4245)
            .field("Channel", format!("<#{channel_id}>"), true)
            .description("*(message not in cache — content unknown)*")
            .timestamp(serenity::Timestamp::now()),
    };

    log_channel
        .send_message(ctx, serenity::CreateMessage::new().embed(embed))
        .await?;
    Ok(())
}

#[allow(clippy::unreadable_literal)]
pub async fn handle_message_update(
    ctx: &serenity::Context,
    data: &crate::Data,
    old_if_available: Option<&serenity::Message>,
    new: Option<&serenity::Message>,
    guild_id: Option<serenity::GuildId>,
) -> Result<(), crate::Error> {
    let Some(guild_id) = guild_id else {
        return Ok(());
    };

    let log_channel = crate::db::get_app_config(&data.db, guild_id)
        .await?
        .and_then(|c| c.mod_log_channel);
    let Some(log_channel) = log_channel else {
        return Ok(());
    };

    let (Some(old), Some(new)) = (old_if_available, new) else {
        return Ok(());
    };
    if old.content == new.content {
        return Ok(());
    }
    if new.author.bot {
        return Ok(());
    }

    let embed = serenity::CreateEmbed::new()
        .title("Message Edited")
        .color(0xFAA61A)
        .field("Author", new.author.tag(), true)
        .field("Channel", format!("<#{}>", new.channel_id), true)
        .field(
            "Before",
            if old.content.is_empty() {
                "*(empty)*".into()
            } else {
                old.content.clone()
            },
            false,
        )
        .field(
            "After",
            if new.content.is_empty() {
                "*(empty)*".into()
            } else {
                new.content.clone()
            },
            false,
        )
        .timestamp(serenity::Timestamp::now());

    log_channel
        .send_message(ctx, serenity::CreateMessage::new().embed(embed))
        .await?;
    Ok(())
}
