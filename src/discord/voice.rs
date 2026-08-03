use std::collections::HashSet;

use serenity::all::{Context, GuildId, UserId};
use tracing::{debug, info};

use crate::db::{self, points, sessions};
use crate::discord::display_name;
use crate::domain::voice::{earning_users, VoiceMember};
use crate::error::{Error, Result};
use crate::state::AppState;

/// Reads the guild's voice state out of the cache into owned values.
///
/// Deliberately not `async`: the cache guard must never be held across an await point.
fn snapshot(ctx: &Context, guild_id: GuildId) -> Option<Vec<VoiceMember>> {
    let guild = ctx.cache.guild(guild_id)?;

    Some(
        guild
            .voice_states
            .values()
            .filter_map(|state| {
                let channel_id = state.channel_id?;
                let is_bot = state
                    .member
                    .as_ref()
                    .map(|member| member.user.bot)
                    .or_else(|| {
                        guild
                            .members
                            .get(&state.user_id)
                            .map(|member| member.user.bot)
                    })
                    .unwrap_or(false);

                Some(VoiceMember {
                    user_id: state.user_id.get(),
                    channel_id: channel_id.get(),
                    is_bot,
                    muted: state.mute || state.self_mute,
                    deafened: state.deaf || state.self_deaf,
                })
            })
            .collect(),
    )
}

/// What a reconciliation changed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Reconciled {
    pub opened: Vec<u64>,
    pub closed: Vec<u64>,
}

/// Brings the open sessions of a whole guild in line with who is actually earning.
///
/// Reconciling the entire guild at once — rather than one channel against the guild's
/// full session list, as the previous version did — is what stops activity in one voice
/// channel from terminating the sessions of everyone in the others.
pub async fn reconcile_guild(
    state: &AppState,
    ctx: &Context,
    guild_id: GuildId,
) -> Result<Reconciled> {
    // Voice events arrive in bursts when a call fills up; serialise per guild so two of
    // them cannot act on the same stale session list.
    let _permit = state.sweep("voice", guild_id).await;

    let members = snapshot(ctx, guild_id).ok_or(Error::GuildNotCached(guild_id.get()))?;
    let earning: HashSet<u64> = earning_users(&members).into_iter().collect();

    let now = db::now(&state.db).await?;
    let open: HashSet<u64> = sessions::active_user_ids(&state.db, guild_id.get())
        .await?
        .into_iter()
        .collect();

    let mut outcome = Reconciled::default();

    for &user_id in earning.difference(&open) {
        // A session credits against a Points row, so the row has to exist first.
        let username = display_name(ctx, guild_id, UserId::new(user_id))
            .await
            .unwrap_or_else(|| user_id.to_string());
        points::ensure_row(&state.db, user_id, guild_id.get(), &username).await?;
        sessions::start(&state.db, user_id, guild_id.get(), now).await?;
        outcome.opened.push(user_id);
        info!(
            user_id,
            guild_id = guild_id.get(),
            username,
            "session opened"
        );
    }

    for &user_id in open.difference(&earning) {
        sessions::finish(&state.db, user_id, guild_id.get(), now).await?;
        outcome.closed.push(user_id);
        info!(user_id, guild_id = guild_id.get(), "session closed");
    }

    debug!(
        guild_id = guild_id.get(),
        earning = earning.len(),
        opened = outcome.opened.len(),
        closed = outcome.closed.len(),
        "voice sessions reconciled"
    );
    Ok(outcome)
}
