use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

pub async fn get_reaction_role(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    message_id: serenity::MessageId,
    emoji: &str,
) -> Result<Option<u64>, sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let message_id_str = message_id.get().to_string();
    let role_id: Option<String> = sqlx::query_scalar!(
        "SELECT role_id FROM reaction_roles WHERE guild_id = ? AND message_id = ? AND emoji = ?",
        guild_id_str,
        message_id_str,
        emoji
    )
    .fetch_optional(pool)
    .await?;

    Ok(role_id.and_then(|r| r.parse().ok()))
}

pub async fn set_reaction_role(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    message_id: serenity::MessageId,
    emoji: &str,
    role_id: u64,
) -> Result<(), sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let message_id_str = message_id.get().to_string();
    let role_id_str = role_id.to_string();
    sqlx::query!(
        "INSERT INTO reaction_roles (guild_id, message_id, emoji, role_id) VALUES (?, ?, ?, ?)
         ON CONFLICT(guild_id, message_id, emoji) DO UPDATE SET role_id = excluded.role_id",
        guild_id_str,
        message_id_str,
        emoji,
        role_id_str
    )
    .execute(pool)
    .await?;
    Ok(())
}