use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

pub async fn init(path: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect(&format!("sqlite://{path}?mode=rwc")).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub struct AppConfig {
    pub mod_log_channel: Option<serenity::ChannelId>,
    pub response_channel: Option<serenity::ChannelId>,
    pub accepted_role: Option<serenity::RoleId>,
}

pub async fn get_app_config(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
) -> Result<Option<AppConfig>, sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let row = sqlx::query!(
        "SELECT mod_log_channel_id, response_channel_id, accepted_role_id FROM app_config WHERE guild_id = ?",
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
    }))
}

pub async fn set_app_config(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    mod_log_channel: Option<serenity::ChannelId>,
    channel: Option<serenity::ChannelId>,
    role: Option<serenity::RoleId>,
) -> Result<(), sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let mod_log_str = mod_log_channel.map(|c| c.get().to_string());
    let channel_str = channel.map(|c| c.get().to_string());
    let role_str = role.map(|r| r.get().to_string());

    sqlx::query!(
        "INSERT INTO app_config (guild_id, mod_log_channel_id, response_channel_id, accepted_role_id) VALUES (?, ?, ?, ?)
         ON CONFLICT(guild_id) DO UPDATE SET
            mod_log_channel_id = COALESCE(excluded.mod_log_channel_id, app_config.mod_log_channel_id),
            response_channel_id = COALESCE(excluded.response_channel_id, app_config.response_channel_id),
            accepted_role_id = COALESCE(excluded.accepted_role_id, app_config.accepted_role_id)",
        guild_id_str,
        mod_log_str,
        channel_str,
        role_str
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Using the struct for the query_as! macro, some stuff is unused...
#[allow(dead_code)]
pub struct Application {
    pub id: i64,
    pub user_id: String,
    pub in_game_name: String,
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
        "SELECT COUNT(*) FROM applications WHERE guild_id = ? AND user_id = ? AND status = 'pending'",
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
    in_game_name: &str,
    answers_json: &str,
) -> Result<i64, sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let user_id_str = user_id.get().to_string();
    let id = sqlx::query!(
        "INSERT INTO applications (guild_id, user_id, in_game_name, answers) VALUES (?, ?, ?, ?)",
        guild_id_str,
        user_id_str,
        in_game_name,
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
) -> Result<Option<Application>, sqlx::Error> {
    sqlx::query_as!(
        Application,
        "SELECT id, user_id, in_game_name, status FROM applications WHERE id = ?",
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
        "UPDATE applications SET status = ?, reviewed_by = ? WHERE id = ?",
        status,
        reviewed_by_str,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_reaction_role(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    key: &str,
) -> Result<Option<u64>, sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let role_id: Option<String> = sqlx::query_scalar!(
        "SELECT role_id FROM reaction_roles WHERE guild_id = ? AND message_key = ?",
        guild_id_str,
        key
    )
    .fetch_optional(pool)
    .await?;

    Ok(role_id.and_then(|r| r.parse().ok()))
}

pub async fn set_reaction_role(
    pool: &SqlitePool,
    guild_id: serenity::GuildId,
    key: &str,
    role_id: u64,
) -> Result<(), sqlx::Error> {
    let guild_id_str = guild_id.get().to_string();
    let role_id_str = role_id.to_string();
    sqlx::query!(
        "INSERT INTO reaction_roles (guild_id, message_key, role_id) VALUES (?, ?, ?)
         ON CONFLICT(guild_id, message_key) DO UPDATE SET role_id = excluded.role_id",
        guild_id_str,
        key,
        role_id_str
    )
    .execute(pool)
    .await?;
    Ok(())
}
