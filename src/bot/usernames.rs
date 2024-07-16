use crate::{db_connection::DbConnection, errors::HelibotError};

use serenity::{
    all::{Context, GuildId, UserId},
    prelude::TypeMapKey,
};
use std::collections::HashMap;
use tokio::sync::Semaphore;

pub struct UsernameManager {
    currently_updating: HashMap<GuildId, Semaphore>,
}
impl TypeMapKey for UsernameManager {
    type Value = UsernameManager;
}

impl UsernameManager {
    pub fn new(guilds: &[GuildId]) -> Self {
        let currently_updating =
            HashMap::from_iter(guilds.iter().map(|g| (g.to_owned(), Semaphore::new(1))));
        UsernameManager { currently_updating }
    }

    pub async fn refresh_usernames(
        &self,
        ctx: &Context,
        guild_id: &GuildId,
    ) -> Result<(), HelibotError> {
        let permit = match self
            .currently_updating
            .get(guild_id)
            .map(|s| s.try_acquire())
        {
            Some(Ok(permit)) => permit,
            _ => {
                println!(
                    "skipping name update bc another one is running for guild {}<{}>",
                    guild_id.name(ctx).unwrap_or("undef".into()),
                    guild_id.get()
                );
                return Ok(());
            }
        };
        let thread_client_data = ctx.data.clone();
        let thread_client_data = thread_client_data.read().await;
        let pool = thread_client_data.get::<DbConnection>().unwrap();
        let points = super::points::get_points_for_guild(ctx, pool, guild_id).await?;
        let members = guild_id.members(&ctx, None, None).await?;
        for point in points {
            let debug = point.username.starts_with("deleted_user_");
            let mut username: String;
            if let Some(member) = members.iter().find(|m| m.user.id == point.user_id) {
                username = member.nick.clone().unwrap_or(member.user.name.clone());
            } else {
                let user = UserId::new(point.user_id).to_user(&ctx).await?;
                username = user.name;
            }
            if username.starts_with("deleted_user_") && !point.username.ends_with(" (deleted user)")
            {
                username = point.username.clone() + " (deleted user)";
            }
            if username != point.username && !username.starts_with("deleted_user_") {
                sqlx::query!(
                    "UPDATE Points SET username=? WHERE user_id=? AND guild_id=?",
                    username,
                    point.user_id,
                    point.guild_id
                )
                .execute(pool)
                .await?;
                println!(
                    "updated username from {} to {} in points",
                    point.username, username
                );
            } else if debug {
                println!("no username change for {username}");
            }
        }
        drop(permit);
        Ok(())
    }
}
