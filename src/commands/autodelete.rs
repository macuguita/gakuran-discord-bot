use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Add a channel to the auto-delete list (only /apply usage allowed there)
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_GUILD",
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn autodelete_add(
    ctx: Context<'_>,
    #[description = "Channel to auto-clean"] channel: serenity::Channel,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in a server.").await?;
        return Ok(());
    };
    crate::db::add_auto_delete_channel(&ctx.data().db, guild_id, channel.id()).await?;
    ctx.say(format!("Messages in <#{}> will now be auto-deleted.", channel.id())).await?;
    Ok(())
}

/// Remove a channel from the auto-delete list
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_GUILD",
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn autodelete_remove(
    ctx: Context<'_>,
    #[description = "Channel to stop auto-cleaning"] channel: serenity::Channel,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in a server.").await?;
        return Ok(());
    };
    crate::db::remove_auto_delete_channel(&ctx.data().db, guild_id, channel.id()).await?;
    ctx.say(format!("Messages in <#{}> will no longer be auto-deleted.", channel.id())).await?;
    Ok(())
}