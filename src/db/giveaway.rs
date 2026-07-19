use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

//Allowing dead code because of the query_as! macro and whatever
#[allow(dead_code)]
pub struct Giveaway {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: String,
    pub message_id: Option<String>,
    pub prize: String,
    pub winner_count: i64,
    pub end_time: i64,
    pub host_id: String,
}

pub async fn insert_giveaway(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
    prize: &str,
    winner_count: i64,
    end_time: i64,
    host_id: serenity::UserId,
) -> Result<i64, sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let channel_id_str = channel_id.get().to_string();
    let host_id_str = host_id.get().to_string();
    let id = sqlx::query!(
        "INSERT INTO giveaways (guild_id, channel_id, prize, winner_count, end_time, host_id)
         VALUES (?, ?, ?, ?, ?, ?)",
        guild_id_str,
        channel_id_str,
        prize,
        winner_count,
        end_time,
        host_id_str
    )
    .execute(pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

pub async fn set_giveaway_message_id(
    pool: &SqlitePool,
    id: i64,
    message_id: serenity::MessageId,
) -> Result<(), sqlx::Error> {
    let message_id_str = message_id.get().to_string();
    sqlx::query!(
        "UPDATE giveaways SET message_id = ? WHERE id = ?",
        message_id_str,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn toggle_entry(
    pool: &SqlitePool,
    giveaway_id: i64,
    user_id: serenity::UserId,
) -> Result<bool, sqlx::Error> {
    // returns true if now entered, false if now removed
    let user_id_str = user_id.get().to_string();
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM giveaway_entries WHERE giveaway_id = ? AND user_id = ?")
            .bind(giveaway_id)
            .bind(&user_id_str)
            .fetch_optional(pool)
            .await?;

    if existing.is_some() {
        sqlx::query!(
            "DELETE FROM giveaway_entries WHERE giveaway_id = ? AND user_id = ?",
            giveaway_id,
            user_id_str
        )
        .execute(pool)
        .await?;
        Ok(false)
    } else {
        sqlx::query!(
            "INSERT INTO giveaway_entries (giveaway_id, user_id) VALUES (?, ?)",
            giveaway_id,
            user_id_str
        )
        .execute(pool)
        .await?;
        Ok(true)
    }
}

pub async fn count_entries(pool: &SqlitePool, giveaway_id: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT COUNT(*) FROM giveaway_entries WHERE giveaway_id = ?",
        giveaway_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_entries(pool: &SqlitePool, giveaway_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT user_id FROM giveaway_entries WHERE giveaway_id = ?",
        giveaway_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.user_id).collect())
}

pub async fn get_due_giveaways(pool: &SqlitePool, now: i64) -> Result<Vec<Giveaway>, sqlx::Error> {
    sqlx::query_as!(
        Giveaway,
        "SELECT id, guild_id, channel_id, message_id, prize, winner_count, end_time, host_id
         FROM giveaways WHERE ended = 0 AND end_time <= ?",
        now
    )
    .fetch_all(pool)
    .await
}

pub async fn mark_giveaway_ended(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("UPDATE giveaways SET ended = 1 WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}
