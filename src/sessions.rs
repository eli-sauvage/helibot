use sqlx::{types::time::OffsetDateTime, MySql, Pool};

use crate::errors::HelibotError;
use crate::points;

#[derive(Debug)]
pub struct ActiveSession {
    id: u32,
    user_id: u64,
    guild_id: u64,
    begin: OffsetDateTime,
}

impl ActiveSession {
    pub async fn get(
        pool: &Pool<MySql>,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Option<ActiveSession>, HelibotError> {
        let session = sqlx::query_as!(
            ActiveSession,
            "SELECT * from ActiveSessions WHERE user_id = ? AND guild_id = ?",
            user_id,
            guild_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(session)
    }
    pub async fn create(
        pool: &Pool<MySql>,
        user_id: u64,
        guild_id: u64,
    ) -> Result<ActiveSession, HelibotError> {
        sqlx::query!("INSERT INTO ActiveSessions (user_id, guild_id, begin) VALUES (?, ?, CURRENT_TIMESTAMP)",
                    user_id,
                    guild_id
                ).execute(pool).await?;
        let session = sqlx::query_as!(
            ActiveSession,
            "SELECT * from ActiveSessions WHERE user_id = ? AND guild_id = ?",
            user_id,
            guild_id
        )
        .fetch_one(pool)
        .await?;
        Ok(session)
    }

    pub async fn terminate(self, pool: &Pool<MySql>) -> Result<(), HelibotError> {
        let now = sqlx::query!("SELECT current_timestamp")
            .fetch_one(pool)
            .await?
            .current_timestamp
            .assume_offset(self.begin.offset());
        let points_to_add = (now - self.begin).whole_seconds();

        let new_points = points::add_points(self.user_id, self.guild_id, points_to_add)?;

        //append in history
        sqlx::query!("INSERT INTO SessionHistory (user_id, guild_id, begin, end, points) VALUES(?, ?, ?, ?, ?)",
            self.user_id,
            self.guild_id,
            self.begin,
            now,
            new_points
        ).execute(pool).await?;

        //delete from active Sessions
        sqlx::query!("DELETE FROM ActiveSessions WHERE id = ?", self.id)
            .execute(pool)
            .await?;

        drop(self);

        Ok(())
    }
}
