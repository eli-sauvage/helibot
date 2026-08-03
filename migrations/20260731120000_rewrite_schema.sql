-- Schema changes for the rewrite. Existing data is preserved; only column names,
-- which were either reserved words or actively misleading, and two new columns change.

-- `last_credited_at` lets voice time be credited incrementally instead of only when a
-- session ends, so a restart loses at most one flush interval rather than every
-- in-flight session.
ALTER TABLE ActiveSessions
    CHANGE `begin` started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;

ALTER TABLE ActiveSessions
    ADD COLUMN last_credited_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP AFTER started_at;

UPDATE ActiveSessions SET last_credited_at = started_at;

-- `points` held the user's running total after the session, not the points earned by the
-- session itself. Renamed to say what it actually contains, and the real per-session
-- figure gets its own column.
ALTER TABLE SessionHistory
    CHANGE `begin` started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;

ALTER TABLE SessionHistory
    CHANGE `end` ended_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;

ALTER TABLE SessionHistory
    CHANGE points total_points_after INT UNSIGNED NOT NULL;

ALTER TABLE SessionHistory
    ADD COLUMN session_seconds INT UNSIGNED NOT NULL DEFAULT 0 AFTER ended_at;

-- Legacy rows never stored the session's own length, but it is recoverable from the
-- timestamps, so the column is correct for the whole history rather than only new rows.
UPDATE SessionHistory
    SET session_seconds = GREATEST(TIMESTAMPDIFF(SECOND, started_at, ended_at), 0)
    WHERE session_seconds = 0;

-- The unique key on Points is (user_id, guild_id), whose leftmost column is user_id, so
-- the per-guild leaderboard query cannot use it.
CREATE INDEX idx_points_guild ON Points (guild_id);
CREATE INDEX idx_session_history_user_guild ON SessionHistory (user_id, guild_id);
