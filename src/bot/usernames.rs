use crate::{bot::points::Point, errors::HelibotError};

use serenity::{
    all::{Context, GuildId, Member, UserId},
    prelude::TypeMapKey,
};
use sqlx::{MySql, Pool};
use std::collections::HashMap;

type UniqueUser = (GuildId, UserId);

#[derive(Default)]
pub struct UsernameManager {
    usernames_cached: HashMap<UniqueUser, String>,
}
impl TypeMapKey for UsernameManager {
    type Value = UsernameManager;
}

impl UsernameManager {
    pub async fn create(
        pool: &Pool<MySql>,
        ctx: &Context,
    ) -> Result<UsernameManager, HelibotError> {
        let points = sqlx::query_as!(Point, "SELECT * from Points")
            .fetch_all(pool)
            .await?;
        let mut valid_users: HashMap<UniqueUser, String> = HashMap::new();

        for point in points {
            let guild_id = GuildId::new(point.guild_id);
            match UserId::new(point.user_id).to_user(ctx).await {
                Ok(user) => {
                    let username = user.nick_in(ctx, guild_id).await.unwrap_or(user.name);
                    valid_users.insert((guild_id, user.id), username);
                }
                Err(e) => {
                    eprintln!(
                        "could not fetch user {} in guild {}. user is in db but not discord guild. Err = {}",
                        point.user_id, point.guild_id, e
                    );
                    eprintln!("deleting it ...");
                    sqlx::query!(
                        "DELETE FROM Points WHERE user_id = ? AND guild_id = ?",
                        point.user_id,
                        point.guild_id
                    )
                    .execute(pool)
                    .await?;
                }
            }
        }
        Ok(UsernameManager {
            usernames_cached: valid_users,
        })
    }

    //pub async fn refresh(&mut self, pool: &Pool<MySql>, ctx: &Context) -> Result<(), HelibotError> {
    //    self.usernames_cached = UsernameManager::create(pool, ctx).await?.usernames_cached;
    //    Ok(())
    //}

    pub async fn add_user(&mut self, member: Member) {
        self.usernames_cached.insert(
            (member.guild_id, member.user.id),
            member.nick.unwrap_or(member.user.name),
        );
    }

    pub fn get_username_from_cache(&self, guild_id: GuildId, user_id: UserId) -> Option<&str> {
        self.usernames_cached
            .get(&(guild_id, user_id))
            .map(|res| res.as_str())
    }
}
