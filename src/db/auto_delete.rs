use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

pub async fn add_auto_delete_channel(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
) -> Result<(), sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let channel_id_str = channel_id.get().to_string();
    sqlx::query!(
        "INSERT OR IGNORE INTO auto_delete_channels (guild_id, channel_id) VALUES (?, ?)",
        guild_id_str,
        channel_id_str
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_auto_delete_channel(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
) -> Result<(), sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let channel_id_str = channel_id.get().to_string();
    sqlx::query!(
        "DELETE FROM auto_delete_channels WHERE guild_id = ? AND channel_id = ?",
        guild_id_str,
        channel_id_str
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn is_auto_delete_channel(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
) -> Result<bool, sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let channel_id_str = channel_id.get().to_string();
    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM auto_delete_channels WHERE guild_id = ? AND channel_id = ?",
        guild_id_str,
        channel_id_str
    )
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}
