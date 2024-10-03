use crate::{errors::HelibotError, models::points};

use serenity::{
    all::{ChannelType, Context, GuildId, Member, Ready, UserId},
    futures::future,
};
use sqlx::{types::time::OffsetDateTime, MySql, Pool};

#[derive(Debug, Clone)]
pub struct ActiveSession {
    id: u32,
    pub user_id: u64,
    pub guild_id: u64,
    pub begin: OffsetDateTime,
}

impl ActiveSession {
    pub async fn terminate(self, ctx: &Context, pool: &Pool<MySql>) -> Result<(), HelibotError> {
        let now = sqlx::query!("SELECT current_timestamp")
            .fetch_one(pool)
            .await?
            .current_timestamp
            .assume_offset(self.begin.offset());
        let points_to_add = (now - self.begin).whole_seconds();

        let new_points = points::add_points(
            ctx,
            pool,
            &UserId::new(self.user_id),
            &GuildId::new(self.guild_id),
            points_to_add,
        )
        .await?;

        sqlx::query!("INSERT INTO SessionHistory (user_id, guild_id, begin, end, points) VALUES(?, ?, ?, ?, ?)",
            self.user_id,
            self.guild_id,
            self.begin,
            now,
            new_points
        ).execute(pool).await?;

        println!("deleting...");
        sqlx::query!("DELETE FROM ActiveSessions WHERE id = ?", self.id)
            .execute(pool)
            .await?;

        Ok(())
    }
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

    pub async fn get_all_active_sessions_for_guild(
        pool: &Pool<MySql>,
        guild_id: u64,
    ) -> Result<Vec<ActiveSession>, HelibotError> {
        let res = sqlx::query_as!(
            ActiveSession,
            "SELECT * from ActiveSessions WHERE guild_id = ?",
            guild_id
        )
        .fetch_all(pool)
        .await?;
        Ok(res)
    }

    pub async fn get_all_active_sessions(
        pool: &Pool<MySql>,
    ) -> Result<Vec<ActiveSession>, HelibotError> {
        let res = sqlx::query_as!(ActiveSession, "SELECT * from ActiveSessions",)
            .fetch_all(pool)
            .await?;
        Ok(res)
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

    pub async fn add_current_sessions_to_db_on_startup(
        pool: &Pool<MySql>,
        ctx: &Context,
        ready: &Ready,
    ) {
        for guild in &ready.guilds {
            let mut connected_members: Vec<Member> = vec![];
            if let Ok(channels) = guild.id.channels(ctx).await {
                channels
                    .values()
                    .flat_map(|guild_channel| match guild_channel.kind {
                        ChannelType::Voice => guild_channel.members(ctx).unwrap_or_default(),
                        _ => {
                            vec![]
                        }
                    })
                    .for_each(|conneted_member| connected_members.push(conneted_member));
            }
            let queries = connected_members.iter().map(|member|async move {
                match sqlx::query!(
                    "INSERT INTO ActiveSessions (user_id, guild_id, begin) VALUES (?, ?, CURRENT_TIMESTAMP)",
                    member.user.id.get(),
                    guild.id.get()
                ).execute(pool).await{
                    Ok(_)=>println!("connected user {} : created session", member.user.name),
                    Err(err)=> eprintln!("could not add active session for user {} on startup : {err:?}", member.user.id.get())
                };
            });
            future::join_all(queries).await;
            // panic!("");
        }
    }
}

pub async fn detect_and_remove_old_sessions(pool: &Pool<MySql>) -> Result<(), HelibotError> {
    sqlx::query!("DELETE FROM ActiveSessions")
        .execute(pool)
        .await?;
    Ok(())
}
