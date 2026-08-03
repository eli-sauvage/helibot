use serenity::all::{Context, EditRole, GuildId, Member, UserId};
use tracing::{debug, info, warn};

use crate::db::{self, points};
use crate::domain::tiers::tier_for;
use crate::error::Result;
use crate::state::{AppState, GuildRole};

/// Resolves every configured tier to a role in the guild, creating the ones that do not
/// exist yet. Cached per guild, so a new guild gets its ladder on join and existing ones
/// cost nothing.
pub async fn ensure_roles(
    state: &AppState,
    ctx: &Context,
    guild_id: GuildId,
) -> Result<Vec<GuildRole>> {
    if let Some(roles) = state.cached_roles(guild_id).await {
        return Ok(roles);
    }

    let guild = guild_id.to_partial_guild(ctx).await?;
    let mut resolved = Vec::with_capacity(state.config.roles.len());

    for tier in &state.config.roles {
        let existing = guild
            .role_by_name(&tier.role_name)
            .map(|role| (role.id, role.name.clone()));

        let (role_id, name) = match existing {
            Some(role) => role,
            None => {
                match guild
                    .create_role(ctx, EditRole::new().name(tier.role_name.clone()))
                    .await
                {
                    Ok(role) => {
                        info!(
                            guild_id = guild_id.get(),
                            role = %tier.role_name,
                            "created missing ladder role"
                        );
                        (role.id, role.name)
                    }
                    Err(err) => {
                        // One missing role must not cost the guild its whole ladder.
                        warn!(
                            guild_id = guild_id.get(),
                            role = %tier.role_name,
                            error = %err,
                            "could not create ladder role, skipping it"
                        );
                        continue;
                    }
                }
            }
        };

        resolved.push(GuildRole {
            role_id,
            name,
            threshold_seconds: tier.threshold_seconds(),
        });
    }

    state.cache_roles(guild_id, resolved.clone()).await;
    Ok(resolved)
}

/// Puts every member of the guild on the one role their score has earned.
pub async fn sync_guild(state: &AppState, ctx: &Context, guild_id: GuildId) -> Result<()> {
    let Some(_permit) = state.try_sweep("roles", guild_id).await else {
        debug!(
            guild_id = guild_id.get(),
            "role sweep already running, skipping"
        );
        return Ok(());
    };

    let roles = ensure_roles(state, ctx, guild_id).await?;
    if roles.is_empty() {
        return Ok(());
    }

    let now = db::now(&state.db).await?;
    let standings = points::standings(&state.db, guild_id.get(), now).await?;
    let members = guild_id.members(ctx, None, None).await?;

    for member in members {
        let Some(standing) = standings
            .iter()
            .find(|standing| standing.user_id == member.user.id.get())
        else {
            continue;
        };

        if let Err(err) = apply(ctx, &member, &roles, standing.points_seconds).await {
            warn!(
                guild_id = guild_id.get(),
                user_id = member.user.id.get(),
                error = %err,
                "could not update roles for a member"
            );
        }
    }

    debug!(guild_id = guild_id.get(), "role sweep done");
    Ok(())
}

/// Same check for a single member, used when their score crosses a threshold mid-session.
pub async fn sync_user(
    state: &AppState,
    ctx: &Context,
    guild_id: GuildId,
    user_id: UserId,
) -> Result<()> {
    let roles = ensure_roles(state, ctx, guild_id).await?;
    if roles.is_empty() {
        return Ok(());
    }

    let now = db::now(&state.db).await?;
    let Some(points_seconds) =
        points::effective_points(&state.db, user_id.get(), guild_id.get(), now).await?
    else {
        return Ok(());
    };

    let member = guild_id.member(ctx, user_id).await?;
    apply(ctx, &member, &roles, points_seconds).await
}

async fn apply(
    ctx: &Context,
    member: &Member,
    roles: &[GuildRole],
    points_seconds: u64,
) -> Result<()> {
    let earned = tier_for(points_seconds, roles);

    let held: Vec<&GuildRole> = roles
        .iter()
        .filter(|role| member.roles.contains(&role.role_id))
        .collect();

    for stale in held
        .iter()
        .filter(|role| earned.is_none_or(|earned| earned.role_id != role.role_id))
    {
        member.remove_role(ctx, stale.role_id).await?;
        info!(
            user_id = member.user.id.get(),
            role = %stale.name,
            "removed role"
        );
    }

    if let Some(earned) = earned {
        if !held.iter().any(|role| role.role_id == earned.role_id) {
            member.add_role(ctx, earned.role_id).await?;
            info!(
                user_id = member.user.id.get(),
                role = %earned.name,
                "granted role"
            );
        }
    }

    Ok(())
}
