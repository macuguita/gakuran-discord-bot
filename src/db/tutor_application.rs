use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

pub struct TutorApplication {
    pub user_id: String,
    pub status: String,
}

pub async fn has_pending_application(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
) -> Result<bool, sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let user_id_str = user_id.get().to_string();
    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tutor_applications WHERE guild_id = ? AND user_id = ? AND status = 'pending'",
        guild_id_str,
        user_id_str
    )
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_application(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    answers_json: &str,
) -> Result<i64, sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let user_id_str = user_id.get().to_string();
    let id = sqlx::query!(
        "INSERT INTO tutor_applications (guild_id, user_id, answers) VALUES (?, ?, ?)",
        guild_id_str,
        user_id_str,
        answers_json
    )
    .execute(pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

pub async fn get_application(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<TutorApplication>, sqlx::Error> {
    sqlx::query_as!(
        TutorApplication,
        "SELECT user_id, status FROM tutor_applications WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn set_application_status(
    pool: &SqlitePool,
    id: i64,
    status: &str,
    reviewed_by: serenity::UserId,
) -> Result<(), sqlx::Error> {
    let reviewed_by_str = reviewed_by.get().to_string();
    sqlx::query!(
        "UPDATE tutor_applications SET status = ?, reviewed_by = ? WHERE id = ?",
        status,
        reviewed_by_str,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}
