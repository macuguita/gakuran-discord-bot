use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Set or disable the channel where deleted/edited messages get logged
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_GUILD",
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn setapplicationresponses(
    ctx: Context<'_>,
    #[description = "Channel to send application responses to (leave blank to disable)"]
    channel: Option<serenity::Channel>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in a server.").await?;
        return Ok(());
    };

    let mut cfg = ctx.data().application_responses.lock().await;

    if let Some(channel) = channel {
        cfg.insert(guild_id, channel.id());
        crate::save_channel_config(&cfg, "application_responses.json")?;
        ctx.say(format!("Application responses set to <#{}>", channel.id()))
            .await?;
    } else {
        cfg.remove(&guild_id);
        crate::save_channel_config(&cfg, "application_responses.json")?;
        ctx.say("Application responses disabled for this server.")
            .await?;
    }

    Ok(())
}
