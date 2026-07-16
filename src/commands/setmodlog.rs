use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Set or disable the channel where deleted/edited messages get logged
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_GUILD",
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn setmodlog(
    ctx: Context<'_>,
    #[description = "Channel to send mod logs to (leave blank to disable)"] channel: Option<
        serenity::Channel,
    >,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in a server.").await?;
        return Ok(());
    };

    if let Some(channel) = channel {
        crate::db::set_mod_log(&ctx.data().db, guild_id, channel.id()).await?;
        ctx.say(format!("Mod log set to <#{}>", channel.id()))
            .await?;
    } else {
        crate::db::remove_mod_log(&ctx.data().db, guild_id).await?;
        ctx.say("Mod log disabled for this server.").await?;
    }

    Ok(())
}