use poise::serenity_prelude as serenity;

/// Set or disable the channel where deleted/edited messages get logged
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_GUILD",
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn setmodlog(
    ctx: crate::Context<'_>,
    #[description = "Channel to send mod logs to (leave blank to disable)"] channel: Option<
        serenity::Channel,
    >,
) -> Result<(), crate::Error> {
    ctx.defer_ephemeral().await?;
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in a server.").await?;
        return Ok(());
    };

    let mut cfg = ctx.data().mod_log.lock().await;

    if let Some(channel) = channel {
        cfg.insert(guild_id, channel.id());
        crate::mod_log::save(&cfg)?;
        ctx.say(format!("Mod log set to <#{}>", channel.id()))
            .await?;
    } else {
        cfg.remove(&guild_id);
        crate::mod_log::save(&cfg)?;
        ctx.say("Mod log disabled for this server.").await?;
    }

    Ok(())
}
