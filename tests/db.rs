//! Database-level tests for the point arithmetic, run against a real MariaDB.
//!
//! `#[sqlx::test]` gives each test its own migrated database, so they need a reachable
//! server: set `DATABASE_URL` before running them. Timestamps are always derived from a
//! single `now` read from the server, which keeps the expected durations exact.

use helibot::db::{points, sessions, Db, DbTime};

const GUILD: u64 = 1;

async fn now(db: &Db) -> DbTime {
    helibot::db::now(db).await.expect("database clock")
}

/// Opens a session that started `age_secs` ago and was last credited
/// `uncredited_secs` ago, relative to `now`.
async fn open_session(db: &Db, user_id: u64, now: DbTime, age_secs: i64, uncredited_secs: i64) {
    sqlx::query("INSERT INTO Points (user_id, guild_id, points, username) VALUES (?, ?, 0, ?)")
        .bind(user_id)
        .bind(GUILD)
        .bind(format!("user{user_id}"))
        .execute(db)
        .await
        .expect("points row");

    sqlx::query(
        "INSERT INTO ActiveSessions (user_id, guild_id, started_at, last_credited_at)
         VALUES (?, ?, ? - INTERVAL ? SECOND, ? - INTERVAL ? SECOND)",
    )
    .bind(user_id)
    .bind(GUILD)
    .bind(now)
    .bind(age_secs)
    .bind(now)
    .bind(uncredited_secs)
    .execute(db)
    .await
    .expect("session row");
}

async fn stored_points(db: &Db, user_id: u64) -> u32 {
    sqlx::query_scalar("SELECT points FROM Points WHERE user_id = ? AND guild_id = ?")
        .bind(user_id)
        .bind(GUILD)
        .fetch_one(db)
        .await
        .expect("points")
}

/// The bug this guards against: the previous version took the first active session's
/// start time and added that same bonus to every connected member.
#[sqlx::test(migrations = "./migrations")]
async fn each_session_is_credited_from_its_own_watermark(db: Db) {
    let now = now(&db).await;
    open_session(&db, 10, now, 600, 120).await;
    open_session(&db, 20, now, 60, 30).await;

    let credited = points::credit_open_sessions(&db, now)
        .await
        .expect("credit sessions");

    assert_eq!(credited, 2);
    assert_eq!(stored_points(&db, 10).await, 120);
    assert_eq!(stored_points(&db, 20).await, 30);
}

/// Same rule, seen through the leaderboard: the live part of a score belongs to the
/// member who earned it.
#[sqlx::test(migrations = "./migrations")]
async fn standings_add_uncredited_time_per_member(db: Db) {
    let now = now(&db).await;
    open_session(&db, 10, now, 600, 120).await;
    open_session(&db, 20, now, 60, 30).await;

    let standings = points::standings(&db, GUILD, now).await.expect("standings");

    let points_of = |user_id: u64| {
        standings
            .iter()
            .find(|standing| standing.user_id == user_id)
            .expect("standing")
            .points_seconds
    };

    assert_eq!(points_of(10), 120);
    assert_eq!(points_of(20), 30);
    assert!(standings.iter().all(|standing| standing.connected));
}

/// Crediting twice in a row must not pay for the same seconds twice.
#[sqlx::test(migrations = "./migrations")]
async fn crediting_is_not_repeated_for_the_same_seconds(db: Db) {
    let now = now(&db).await;
    open_session(&db, 10, now, 600, 120).await;

    points::credit_open_sessions(&db, now).await.expect("first");
    points::credit_open_sessions(&db, now)
        .await
        .expect("second");

    assert_eq!(stored_points(&db, 10).await, 120);
}

/// `session_seconds` is what the session was worth; `total_points_after` is the running
/// total. The previous version wrote the running total into a column named `points`.
#[sqlx::test(migrations = "./migrations")]
async fn finishing_records_the_session_and_the_running_total(db: Db) {
    let now = now(&db).await;
    open_session(&db, 10, now, 300, 60).await;
    // 240 of the 300 seconds were already banked by earlier credit passes.
    sqlx::query("UPDATE Points SET points = 240 WHERE user_id = 10")
        .execute(&db)
        .await
        .expect("preset points");

    sessions::finish(&db, 10, GUILD, now).await.expect("finish");

    assert_eq!(stored_points(&db, 10).await, 300);

    let (session_seconds, total_after): (u32, u32) = sqlx::query_as(
        "SELECT session_seconds, total_points_after FROM SessionHistory WHERE user_id = ?",
    )
    .bind(10u64)
    .fetch_one(&db)
    .await
    .expect("history row");

    assert_eq!(session_seconds, 300, "the session's own length");
    assert_eq!(total_after, 300, "the member's total afterwards");

    let open: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ActiveSessions")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(open, 0);
}

/// Finishing a session that is not open is a no-op, not an error: reconciliation can
/// race with itself when voice events arrive in bursts.
#[sqlx::test(migrations = "./migrations")]
async fn finishing_an_unknown_session_is_harmless(db: Db) {
    let now = now(&db).await;
    sessions::finish(&db, 999, GUILD, now)
        .await
        .expect("no session");

    let history: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM SessionHistory")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(history, 0);
}

/// Startup keeps the time a crashed run had already banked, instead of the previous
/// version's bare `DELETE FROM ActiveSessions`.
#[sqlx::test(migrations = "./migrations")]
async fn startup_archives_leftover_sessions_at_their_watermark(db: Db) {
    let now = now(&db).await;
    open_session(&db, 10, now, 600, 120).await;
    points::credit_open_sessions(&db, now)
        .await
        .expect("credit before the crash");

    let archived = sessions::archive_orphans(&db).await.expect("archive");

    assert_eq!(archived, 1);
    assert_eq!(
        stored_points(&db, 10).await,
        120,
        "credited time survives the restart"
    );

    let session_seconds: u32 =
        sqlx::query_scalar("SELECT session_seconds FROM SessionHistory WHERE user_id = 10")
            .fetch_one(&db)
            .await
            .expect("history row");
    // Started 600s ago, credited up to `now`: the archived length runs to the watermark.
    assert_eq!(session_seconds, 600);

    let open: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ActiveSessions")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(open, 0);
}

/// Two voice events for the same member must not open two sessions.
#[sqlx::test(migrations = "./migrations")]
async fn starting_twice_opens_one_session(db: Db) {
    let now = now(&db).await;
    points::ensure_row(&db, 10, GUILD, "someone")
        .await
        .expect("points row");

    sessions::start(&db, 10, GUILD, now).await.expect("first");
    sessions::start(&db, 10, GUILD, now).await.expect("second");

    let open = sessions::active_user_ids(&db, GUILD).await.expect("open");
    assert_eq!(open, vec![10]);
}

/// `ensure_row` must not overwrite a name the username sweep has already resolved.
#[sqlx::test(migrations = "./migrations")]
async fn ensuring_a_row_keeps_an_existing_username(db: Db) {
    points::ensure_row(&db, 10, GUILD, "real name")
        .await
        .expect("first");
    points::ensure_row(&db, 10, GUILD, "10")
        .await
        .expect("second");

    let username: String =
        sqlx::query_scalar("SELECT username FROM Points WHERE user_id = 10 AND guild_id = ?")
            .bind(GUILD)
            .fetch_one(&db)
            .await
            .expect("username");
    assert_eq!(username, "real name");
}
