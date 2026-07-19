use crate::Context;
use crate::db::appconfig::{get_app_config, set_app_config};
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Configure applications, accepted role, and mod log channel
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_GUILD",
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn appconfig(
    ctx: Context<'_>,
    #[description = "Channel where mesage modifications get logged"] mod_log: Option<
        serenity::Channel,
    >,
    #[description = "Channel where applications get sent for review"] application_channel: Option<
        serenity::Channel,
    >,
    #[description = "Role given when an application is accepted"] application_approved_role: Option<
        serenity::Role,
    >,
    #[description = "Channel where tutor applications get sent for review"]
    tutor_application_channel: Option<serenity::Channel>,
    #[description = "Role given when a tutor application is accepted"]
    tutor_application_approved_role: Option<serenity::Role>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in a server.").await?;
        return Ok(());
    };

    if application_channel.is_none() && application_approved_role.is_none() && mod_log.is_none() {
        ctx.say("Provide at least a channel, a role, or a mod log channel to update.")
            .await?;
        return Ok(());
    }

    set_app_config(
        &ctx.data().db,
        guild_id,
        mod_log.as_ref().map(serenity::Channel::id),
        application_channel.as_ref().map(serenity::Channel::id),
        application_approved_role.as_ref().map(|r| r.id),
        tutor_application_channel
            .as_ref()
            .map(serenity::Channel::id),
        tutor_application_approved_role.as_ref().map(|r| r.id),
    )
    .await?;

    let cfg = get_app_config(&ctx.data().db, guild_id).await?;
    let (mod_log_str, app_chan_str, app_role_str, tutor_app_channel_str, tutor_app_role_str) = cfg
        .map_or(
            (
                "*(not set)*".into(),
                "*(not set)*".into(),
                "*(not set)*".into(),
                "*(not set)*".into(),
                "*(not set)*".into(),
            ),
            |c| {
                (
                    c.mod_log_channel
                        .map_or("*(not set)*".into(), |c| format!("<#{c}>")),
                    c.response_channel
                        .map_or("*(not set)*".into(), |c| format!("<#{c}>")),
                    c.accepted_role
                        .map_or("*(not set)*".into(), |r| format!("<@&{r}>")),
                    c.tutor_response_channel
                        .map_or("*(not set)*".into(), |c| format!("<#{c}>")),
                    c.tutor_accepted_role
                        .map_or("*(not set)*".into(), |r| format!("<@&{r}>")),
                )
            },
        );

    ctx.say(format!(
        r"Mod log channel: {mod_log_str}
Application channel: {app_chan_str}
Accepted role: {app_role_str}
Tutor application channel: {tutor_app_channel_str}
Tutor accepted role: {tutor_app_role_str}"
    ))
    .await?;
    Ok(())
}
