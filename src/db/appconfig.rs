use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

pub struct AppConfig {
    pub mod_log_channel: Option<serenity::ChannelId>,
    pub response_channel: Option<serenity::ChannelId>,
    pub accepted_role: Option<serenity::RoleId>,
    pub tutor_response_channel: Option<serenity::ChannelId>,
    pub tutor_accepted_role: Option<serenity::RoleId>,
}

pub async fn get_app_config(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
) -> Result<Option<AppConfig>, sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let row = sqlx::query!(
        "SELECT mod_log_channel_id, response_channel_id, accepted_role_id, tutor_response_channel_id, tutor_accepted_role_id FROM guild_config WHERE guild_id = ?",
        guild_id_str
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| AppConfig {
        mod_log_channel: r
            .mod_log_channel_id
            .and_then(|c| c.parse().ok())
            .map(serenity::ChannelId::new),
        response_channel: r
            .response_channel_id
            .and_then(|c| c.parse().ok())
            .map(serenity::ChannelId::new),
        accepted_role: r
            .accepted_role_id
            .and_then(|r| r.parse().ok())
            .map(serenity::RoleId::new),
        tutor_response_channel: r
            .tutor_response_channel_id
            .and_then(|c| c.parse().ok())
            .map(serenity::ChannelId::new),
        tutor_accepted_role: r
            .tutor_accepted_role_id
            .and_then(|r| r.parse().ok())
            .map(serenity::RoleId::new),
    }))
}

pub async fn set_app_config(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    mod_log_channel: Option<serenity::ChannelId>,
    response_channel: Option<serenity::ChannelId>,
    accepted_role: Option<serenity::RoleId>,
    tutor_response_channel: Option<serenity::ChannelId>,
    tutor_accepted_role: Option<serenity::RoleId>,
) -> Result<(), sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let mod_log_str = mod_log_channel.map(|c| c.get().to_string());
    let response_channel_str = response_channel.map(|c| c.get().to_string());
    let accepted_role_str = accepted_role.map(|r| r.get().to_string());
    let tutor_response_channel_str = tutor_response_channel.map(|c| c.get().to_string());
    let tutor_accepted_role_str = tutor_accepted_role.map(|r| r.get().to_string());

    sqlx::query!(
        "INSERT INTO guild_config (guild_id, mod_log_channel_id, response_channel_id, accepted_role_id, tutor_response_channel_id, tutor_accepted_role_id) VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(guild_id) DO UPDATE SET
            mod_log_channel_id = COALESCE(excluded.mod_log_channel_id, guild_config.mod_log_channel_id),
            response_channel_id = COALESCE(excluded.response_channel_id, guild_config.response_channel_id),
            accepted_role_id = COALESCE(excluded.accepted_role_id, guild_config.accepted_role_id),
            tutor_response_channel_id = COALESCE(excluded.tutor_response_channel_id, guild_config.tutor_response_channel_id),
            tutor_accepted_role_id = COALESCE(excluded.tutor_accepted_role_id, guild_config.tutor_accepted_role_id)",
        guild_id_str,
        mod_log_str,
        response_channel_str,
        accepted_role_str,
        tutor_response_channel_str,
        tutor_accepted_role_str,
    )
    .execute(pool)
    .await?;
    Ok(())
}
