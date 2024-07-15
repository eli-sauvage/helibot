use serde::Deserialize;
use serenity::{
    all::{Context, EditRole, GuildId, Role, UserId},
    prelude::TypeMapKey,
};
use sqlx::{MySql, Pool};
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::errors::HelibotError;

use super::points::get_points_for_user;

#[derive(Debug, Deserialize, Clone)]
pub struct Seuil {
    pub seuil: u32,
    pub role_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct RoleWithSeuil {
    role: Role,
    seuil: u32,
}
pub struct RoleManager {
    seuils: Vec<Seuil>,
    roles: RwLock<HashMap<GuildId, Vec<RoleWithSeuil>>>,
}
impl TypeMapKey for RoleManager {
    type Value = RoleManager;
}

impl RoleManager {
    pub async fn new(
        ctx: &Context,
        guild_ids: &Vec<GuildId>,
        seuils: Vec<Seuil>,
    ) -> Result<Self, HelibotError> {
        let mut roles: HashMap<GuildId, Vec<RoleWithSeuil>> = HashMap::new();

        for guild_id in guild_ids {
            let guild = guild_id.to_partial_guild(ctx).await?;
            let mut roles_for_guild: Vec<RoleWithSeuil> = vec![];
            for seuil in &seuils {
                if let Some(r) = guild.role_by_name(&seuil.role_name) {
                    roles_for_guild.push(RoleWithSeuil {
                        role: r.to_owned(),
                        seuil: seuil.seuil,
                    });
                } else {
                    let r = EditRole::new().name(seuil.role_name.to_owned());
                    roles_for_guild.push(RoleWithSeuil {
                        role: guild.create_role(ctx, r).await?,
                        seuil: seuil.seuil,
                    });
                }
            }
            roles.insert(guild_id.to_owned(), roles_for_guild);
        }

        println!(
            "instanciate role w/ seuils : {}",
            seuils
                .iter()
                .map(|s| format!("{}:{}", s.role_name, s.seuil))
                .collect::<Vec<_>>()
                .join(", ")
        );

        Ok(RoleManager {
            seuils,
            roles: RwLock::new(roles),
        })
    }

    async fn get_roles_for_guild(
        &self,
        ctx: &Context,
        guild_id: &GuildId,
    ) -> Result<Vec<RoleWithSeuil>, HelibotError> {
        if let Some(roles) = self.roles.read().await.get(guild_id) {
            Ok(roles.to_vec())
        } else {
            let guild = guild_id.to_partial_guild(ctx).await?;
            let mut roles_for_guild: Vec<RoleWithSeuil> = vec![];
            for seuil in &self.seuils {
                if let Some(r) = guild.role_by_name(&seuil.role_name) {
                    roles_for_guild.push(RoleWithSeuil {
                        role: r.to_owned(),
                        seuil: seuil.seuil,
                    });
                } else {
                    let r = EditRole::new().name(seuil.role_name.to_owned());
                    roles_for_guild.push(RoleWithSeuil {
                        role: guild.create_role(ctx, r).await?,
                        seuil: seuil.seuil,
                    });
                }
            }
            let mut roles = self.roles.write().await;
            roles.insert(guild_id.to_owned(), roles_for_guild.clone());
            Ok(roles_for_guild)
        }
    }

    pub fn get_seuils(&self) -> &Vec<Seuil> {
        &self.seuils
    }

    pub async fn check_role_for_user(
        &self,
        pool: &Pool<MySql>,
        user_id: &UserId,
        guild_id: &GuildId,
        ctx: &Context,
    ) -> Result<(), HelibotError> {
        let helibot_roles = self.get_roles_for_guild(ctx, guild_id).await?;
        if helibot_roles.is_empty() {
            return Ok(());
        }
        /*let helibot_min_role = helibot_roles
        .iter()
        .min_by(|a, b| a.seuil.cmp(&b.seuil))
        .unwrap();*/
        let point = match get_points_for_user(pool, user_id, guild_id).await? {
            Some(p) => p,
            None => return Ok(()),
        };

        //let user = user_id.to_user(ctx).await?;
        let guild = guild_id.to_partial_guild(ctx).await?;
        let member = guild.member(&ctx.http, user_id).await?; //using http to force refetch
        let helibot_roles = self.get_roles_for_guild(ctx, guild_id).await?;

        let roles_user: Vec<&RoleWithSeuil> = member
            .roles
            .iter()
            .filter_map(|r| {
                helibot_roles
                    .iter()
                    .find(|role_seuil| &role_seuil.role.id == r)
            })
            .collect();

        let computed_role = helibot_roles
            .iter()
            .reduce(|acc, role| {
                if point.points >= role.seuil {
                    role
                } else {
                    acc
                }
            })
            .unwrap();

        for role_to_remove in roles_user.iter().filter(|r| **r != computed_role) {
            member.remove_role(&ctx, role_to_remove.role.id).await?;
            println!(
                "removed role {} for user {:?}",
                role_to_remove.role.name, &member.user.name
            );
        }
        if !roles_user.contains(&computed_role) {
            member.add_role(&ctx, computed_role.role.id).await?;
            println!(
                "added role {} for user {:?}",
                computed_role.role.name, &member.user.name
            );
        }

        Ok(())
    }
}
