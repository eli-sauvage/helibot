use std::collections::HashSet;

use serenity::all::{Context, GuildId, UserId};
use tracing::{debug, info, warn};

use crate::db::{self, points};
use crate::error::Result;
use crate::state::AppState;

/// Marks accounts that no longer exist, so the board does not fill up with
/// `deleted_user_…` placeholders.
const DELETED_PREFIX: &str = "deleted_user_";
const DELETED_SUFFIX: &str = " (deleted user)";

/// Reconciles stored usernames with Discord, and refreshes the member list the board
/// uses to hide people who have left.
pub async fn sync_guild(state: &AppState, ctx: &Context, guild_id: GuildId) -> Result<()> {
    let Some(_permit) = state.try_sweep("usernames", guild_id).await else {
        debug!(
            guild_id = guild_id.get(),
            "username sweep already running, skipping"
        );
        return Ok(());
    };

    let members = guild_id.members(ctx, None, None).await?;
    let member_ids: HashSet<u64> = members.iter().map(|member| member.user.id.get()).collect();
    state.cache_members(guild_id, member_ids).await;

    let now = db::now(&state.db).await?;
    let standings = points::standings(&state.db, guild_id.get(), now).await?;

    for standing in standings {
        let current = members
            .iter()
            .find(|member| member.user.id.get() == standing.user_id)
            .map(|member| member.display_name().to_owned());

        let desired = match current {
            Some(name) => name,
            None => {
                // Already tagged: no need to ask Discord about them again.
                if standing.username.ends_with(DELETED_SUFFIX) {
                    continue;
                }
                match deleted_account_name(ctx, standing.user_id, &standing.username).await {
                    Some(name) => name,
                    None => continue,
                }
            }
        };

        if desired == standing.username || desired.starts_with(DELETED_PREFIX) {
            continue;
        }

        points::update_username(&state.db, standing.user_id, guild_id.get(), &desired).await?;
        info!(
            user_id = standing.user_id,
            from = %standing.username,
            to = %desired,
            "username updated"
        );
    }

    debug!(guild_id = guild_id.get(), "username sweep done");
    Ok(())
}

/// Tags a member who has left, once, if Discord reports the account as deleted. Anyone
/// who simply left the guild keeps the name they had.
async fn deleted_account_name(ctx: &Context, user_id: u64, stored: &str) -> Option<String> {
    match UserId::new(user_id).to_user(ctx).await {
        Ok(user) if user.name.starts_with(DELETED_PREFIX) => {
            Some(format!("{stored}{DELETED_SUFFIX}"))
        }
        Ok(_) => None,
        Err(err) => {
            warn!(user_id, error = %err, "could not look up a former member");
            None
        }
    }
}
