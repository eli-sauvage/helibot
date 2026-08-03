use crate::db::{Db, DbTime};
use crate::error::Result;

pub async fn active_user_ids(db: &Db, guild_id: u64) -> Result<Vec<u64>> {
    let ids = sqlx::query_scalar!(
        "SELECT user_id FROM ActiveSessions WHERE guild_id = ?",
        guild_id
    )
    .fetch_all(db)
    .await?;
    Ok(ids)
}

/// Opens a session. The unique key on `(user_id, guild_id)` makes this safe to call
/// twice for the same member.
pub async fn start(db: &Db, user_id: u64, guild_id: u64, now: DbTime) -> Result<()> {
    sqlx::query!(
        "INSERT IGNORE INTO ActiveSessions (user_id, guild_id, started_at, last_credited_at) VALUES (?, ?, ?, ?)",
        user_id,
        guild_id,
        now,
        now
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Closes a session: credits the remainder, archives it, and removes it.
///
/// `SessionHistory.session_seconds` is what this session was worth;
/// `total_points_after` is the member's running total. The previous version wrote the
/// running total into a column called `points`, which made the table unusable for
/// anything per-session.
pub async fn finish(db: &Db, user_id: u64, guild_id: u64, now: DbTime) -> Result<()> {
    let mut tx = db.begin().await?;

    let session = sqlx::query!(
        "SELECT id, started_at, last_credited_at FROM ActiveSessions WHERE user_id = ? AND guild_id = ? FOR UPDATE",
        user_id,
        guild_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(session) = session else {
        tx.rollback().await?;
        return Ok(());
    };

    sqlx::query!(
        r#"
        UPDATE Points
           SET points = points + GREATEST(TIMESTAMPDIFF(SECOND, ?, ?), 0)
         WHERE user_id = ? AND guild_id = ?
        "#,
        session.last_credited_at,
        now,
        user_id,
        guild_id
    )
    .execute(&mut *tx)
    .await?;

    let total = sqlx::query_scalar!(
        "SELECT points FROM Points WHERE user_id = ? AND guild_id = ?",
        user_id,
        guild_id
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO SessionHistory (user_id, guild_id, started_at, ended_at, session_seconds, total_points_after)
        VALUES (?, ?, ?, ?, GREATEST(TIMESTAMPDIFF(SECOND, ?, ?), 0), ?)
        "#,
        user_id,
        guild_id,
        session.started_at,
        now,
        session.started_at,
        now,
        total
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM ActiveSessions WHERE id = ?", session.id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// Clears sessions left behind by a previous run.
///
/// Time is credited continuously, so whatever those sessions were worth is already in
/// `Points` up to `last_credited_at`; they are archived at that watermark and dropped.
/// The previous version issued a bare `DELETE FROM ActiveSessions` at startup, throwing
/// away every in-flight session in full. Returns how many were archived.
pub async fn archive_orphans(db: &Db) -> Result<u64> {
    let mut tx = db.begin().await?;

    let archived = sqlx::query!(
        r#"
        INSERT INTO SessionHistory (user_id, guild_id, started_at, ended_at, session_seconds, total_points_after)
        SELECT a.user_id,
               a.guild_id,
               a.started_at,
               a.last_credited_at,
               GREATEST(TIMESTAMPDIFF(SECOND, a.started_at, a.last_credited_at), 0),
               COALESCE(p.points, 0)
          FROM ActiveSessions a
          LEFT JOIN Points p
                 ON p.user_id = a.user_id AND p.guild_id = a.guild_id
         WHERE a.last_credited_at > a.started_at
        "#
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    sqlx::query!("DELETE FROM ActiveSessions")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(archived)
}
