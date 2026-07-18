pub mod appconfig;
pub mod application;
pub mod auto_delete;
pub mod reaction_roles;

use sqlx::SqlitePool;

pub async fn init(path: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect(&format!("sqlite://{path}?mode=rwc")).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
