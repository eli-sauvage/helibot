use std::time::Duration;

use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use time::PrimitiveDateTime;
use tracing::info;

use crate::error::Result;

pub mod points;
pub mod sessions;

pub type Db = Pool<MySql>;

/// Timestamps come from the database, never from the process clock.
///
/// MySQL stores `TIMESTAMP` in UTC but renders it in the session time zone, so mixing a
/// process-side "now" with a column value silently depends on two clocks agreeing. Every
/// duration in this bot is computed by the server, from a single `now` bound as a
/// parameter, which is what `PrimitiveDateTime` carries.
pub type DbTime = PrimitiveDateTime;

pub async fn connect_and_migrate(database_url: &str) -> Result<Db> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("database ready, migrations up to date");

    Ok(pool)
}

/// The database's clock, used as the single reference point for a unit of work.
pub async fn now(db: &Db) -> Result<DbTime> {
    let now = sqlx::query_scalar!(r#"SELECT CURRENT_TIMESTAMP AS "now!: DbTime""#)
        .fetch_one(db)
        .await?;
    Ok(now)
}
