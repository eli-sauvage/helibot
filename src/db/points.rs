use crate::db::{Db, DbTime};
use crate::error::Result;

/// A member's standing, including the part of an open session that has not been
/// credited yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standing {
    pub user_id: u64,
    pub username: String,
    pub points_seconds: u64,
    pub connected: bool,
}

/// Creates the row a session needs to credit against. Existing rows, and the usernames
/// in them, are left alone.
pub async fn ensure_row(db: &Db, user_id: u64, guild_id: u64, username: &str) -> Result<()> {
    sqlx::query!(
        "INSERT IGNORE INTO Points (user_id, guild_id, points, username) VALUES (?, ?, 0, ?)",
        user_id,
        guild_id,
        username
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Every member of a guild with their effective score, unsorted.
///
/// The live part of the score is computed per session from that session's own
/// `last_credited_at`. The previous version took the first active session's start time
/// and added the same bonus to everybody.
pub async fn standings(db: &Db, guild_id: u64, now: DbTime) -> Result<Vec<Standing>> {
    let rows = sqlx::query!(
        r#"
        SELECT p.user_id,
               p.username,
               CAST(p.points + COALESCE(GREATEST(TIMESTAMPDIFF(SECOND, a.last_credited_at, ?), 0), 0) AS UNSIGNED) AS "points_seconds!: u64",
               a.id AS "session_id?: u32"
        FROM Points p
        LEFT JOIN ActiveSessions a
               ON a.user_id = p.user_id AND a.guild_id = p.guild_id
        WHERE p.guild_id = ?
        "#,
        now,
        guild_id
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Standing {
            user_id: row.user_id,
            username: row.username,
            points_seconds: row.points_seconds,
            connected: row.session_id.is_some(),
        })
        .collect())
}

/// A single member's effective score, for a targeted role check.
pub async fn effective_points(
    db: &Db,
    user_id: u64,
    guild_id: u64,
    now: DbTime,
) -> Result<Option<u64>> {
    let points = sqlx::query_scalar!(
        r#"
        SELECT CAST(p.points + COALESCE(GREATEST(TIMESTAMPDIFF(SECOND, a.last_credited_at, ?), 0), 0) AS UNSIGNED) AS "points_seconds!: u64"
        FROM Points p
        LEFT JOIN ActiveSessions a
               ON a.user_id = p.user_id AND a.guild_id = p.guild_id
        WHERE p.user_id = ? AND p.guild_id = ?
        "#,
        now,
        user_id,
        guild_id
    )
    .fetch_optional(db)
    .await?;

    Ok(points)
}

/// Banks the voice time accumulated since the last pass and moves the watermark.
///
/// Both statements share one `now` so no slice of time can fall between them, and
/// `GREATEST(..., 0)` keeps a backwards clock from underflowing an unsigned column.
/// Returns how many members were credited.
pub async fn credit_open_sessions(db: &Db, now: DbTime) -> Result<u64> {
    let mut tx = db.begin().await?;

    let credited = sqlx::query!(
        r#"
        UPDATE Points p
          JOIN ActiveSessions a
            ON a.user_id = p.user_id AND a.guild_id = p.guild_id
           SET p.points = p.points + GREATEST(TIMESTAMPDIFF(SECOND, a.last_credited_at, ?), 0)
        "#,
        now
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    sqlx::query!("UPDATE ActiveSessions SET last_credited_at = ?", now)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(credited)
}

pub async fn update_username(db: &Db, user_id: u64, guild_id: u64, username: &str) -> Result<()> {
    sqlx::query!(
        "UPDATE Points SET username = ? WHERE user_id = ? AND guild_id = ?",
        username,
        user_id,
        guild_id
    )
    .execute(db)
    .await?;
    Ok(())
}
