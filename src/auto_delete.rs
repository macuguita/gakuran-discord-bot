use crate::db::auto_delete::is_auto_delete_channel;
use anyhow::Result;
use poise::serenity_prelude as serenity;

pub async fn handle_message(
    ctx: &serenity::Context,
    data: &crate::Data,
    msg: &serenity::Message,
) -> Result<()> {
    // Don't try to delete the bot's own messages, and skip DMs
    if msg.author.bot {
        return Ok(());
    }
    let Some(guild_id) = msg.guild_id else {
        return Ok(());
    };

    if is_auto_delete_channel(&data.db, guild_id, msg.channel_id).await? {
        // The message itself can be a command like /apply (slash commands don't
        // show up as regular messages) or accidental chatter — either way, delete it
        msg.delete(ctx).await?;
    }

    Ok(())
}
