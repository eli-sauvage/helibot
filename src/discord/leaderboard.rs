use serenity::all::{
    ChannelId, Context, CreateEmbed, CreateEmbedFooter, CreateMessage, EditMessage, GetMessages,
    GuildId, MessageId, Timestamp,
};
use tracing::{debug, info, warn};

use crate::db::{self, points};
use crate::domain::leaderboard::{fields, Row, TOP_N};
use crate::error::{Error, Result};
use crate::state::{AppState, PostedBoard};

/// Finds the channel the board lives in: the configured id when there is one, otherwise
/// the channel named `point_channel_name`. The result is cached, so the name lookup
/// costs one request per process rather than one per refresh.
async fn resolve_channel(state: &AppState, ctx: &Context, guild_id: GuildId) -> Result<ChannelId> {
    if let Some(channel_id) = state.config.points_channel_id(guild_id.get()) {
        return Ok(ChannelId::new(channel_id));
    }

    if let Some(channel_id) = state.cached_channel(guild_id).await {
        return Ok(channel_id);
    }

    let channels = guild_id.channels(ctx).await?;
    let found = channels
        .into_iter()
        .find(|(_, channel)| channel.name == state.config.point_channel_name)
        .map(|(id, _)| id)
        .ok_or(Error::PointsChannelNotFound(guild_id.get()))?;

    state.cache_channel(guild_id, found).await;
    Ok(found)
}

/// Rebuilds the board and edits the existing message, posting a new one only if the edit
/// fails or nothing has been posted yet.
///
/// The channel must end up holding exactly one message. Refreshes are serialised per
/// guild because several of them start at once: `guild_create` triggers one, and the
/// refresh loop's first tick fires immediately. Running concurrently, both would find no
/// board recorded and both would post.
pub async fn refresh(state: &AppState, ctx: &Context, guild_id: GuildId) -> Result<()> {
    let _permit = state.sweep("board", guild_id).await;

    let channel_id = resolve_channel(state, ctx, guild_id).await?;

    let now = db::now(&state.db).await?;
    let mut standings = points::standings(&state.db, guild_id.get(), now).await?;

    // Hide people who have left the guild, when a member list is known. It is refreshed
    // by the username sweep rather than fetched here, so the board costs no member
    // request of its own.
    if let Some(members) = state.cached_members(guild_id).await {
        standings.retain(|standing| members.contains(&standing.user_id));
    }

    standings.sort_by(|a, b| {
        b.points_seconds
            .cmp(&a.points_seconds)
            .then_with(|| a.username.cmp(&b.username))
    });

    let rows: Vec<Row> = standings
        .into_iter()
        .map(|standing| Row {
            username: standing.username,
            points_seconds: standing.points_seconds,
            connected: standing.connected,
        })
        .collect();

    let fields = fields(&rows);
    let embed = build_embed(state, fields.clone());

    if let Some(board) = state.board(guild_id).await {
        if board.channel_id == channel_id && board.fields == fields {
            debug!(guild_id = guild_id.get(), "board unchanged, skipping edit");
            return Ok(());
        }

        let edited = board
            .channel_id
            .edit_message(
                ctx,
                board.message_id,
                EditMessage::new().embed(embed.clone()),
            )
            .await;

        match edited {
            Ok(_) => {
                state
                    .set_board(
                        guild_id,
                        PostedBoard {
                            channel_id: board.channel_id,
                            message_id: board.message_id,
                            fields,
                        },
                    )
                    .await;
                return Ok(());
            }
            Err(err) => {
                warn!(
                    guild_id = guild_id.get(),
                    error = %err,
                    "could not edit the board, posting a new one"
                );
                state.forget_board(guild_id).await;
            }
        }
    }

    claim_channel(ctx, channel_id).await;

    let message = channel_id
        .send_message(ctx, CreateMessage::new().embed(embed))
        .await?;

    state
        .set_board(
            guild_id,
            PostedBoard {
                channel_id,
                message_id: message.id,
                fields,
            },
        )
        .await;

    info!(
        guild_id = guild_id.get(),
        channel_id = channel_id.get(),
        "posted a new board"
    );
    Ok(())
}

fn build_embed(state: &AppState, fields: Vec<(String, String, bool)>) -> CreateEmbed {
    let footer = CreateEmbedFooter::new(
        state
            .config
            .roles
            .iter()
            .map(|tier| format!("{} : {}", tier.role_name, tier.seuil))
            .collect::<Vec<_>>()
            .join(", "),
    );

    let minutes = state.config.refresh_interval_secs.div_ceil(60).max(1);

    CreateEmbed::new()
        .title("Helibot scores. 1 minute en vocal = 1 point")
        .description(format!(
            "Score des {TOP_N} premiers. Le message s'update toutes les {minutes} minutes"
        ))
        .footer(footer)
        .fields(fields)
        .timestamp(Timestamp::now())
}

/// Puts a new board back after the old one is deleted, so the channel is never left
/// empty. Ignores anything that is not the message currently being maintained —
/// including the bot's own housekeeping deletions.
pub async fn repost_if_deleted(
    state: &AppState,
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
    deleted: &[MessageId],
) {
    let Some(board) = state.board(guild_id).await else {
        return;
    };
    if board.channel_id != channel_id || !deleted.contains(&board.message_id) {
        return;
    }

    info!(
        guild_id = guild_id.get(),
        message_id = board.message_id.get(),
        "board was deleted, posting a new one"
    );
    state.forget_board(guild_id).await;

    if let Err(err) = refresh(state, ctx, guild_id).await {
        warn!(guild_id = guild_id.get(), error = %err, "could not repost the board");
    }
}

/// Empties the board channel before a new board is posted.
///
/// The channel is dedicated to this one message, so anything else in it — boards from
/// earlier runs, a duplicate from a concurrent post — is cleared rather than left to
/// accumulate.
async fn claim_channel(ctx: &Context, channel_id: ChannelId) {
    loop {
        let messages = match channel_id
            .messages(ctx, GetMessages::new().limit(100))
            .await
        {
            Ok(messages) => messages,
            Err(err) => {
                warn!(channel_id = channel_id.get(), error = %err, "could not list old messages");
                return;
            }
        };

        if messages.is_empty() {
            return;
        }

        let ids: Vec<MessageId> = messages.iter().map(|message| message.id).collect();
        let batch = ids.len();

        // Bulk delete needs at least two messages and refuses anything older than two
        // weeks, so both of those cases fall back to deleting one at a time.
        let bulk = if batch > 1 {
            channel_id.delete_messages(ctx, &ids).await
        } else {
            channel_id.delete_message(ctx, ids[0]).await
        };

        if let Err(err) = bulk {
            debug!(channel_id = channel_id.get(), error = %err, "bulk delete refused, deleting individually");
            for id in ids {
                if let Err(err) = channel_id.delete_message(ctx, id).await {
                    warn!(
                        channel_id = channel_id.get(),
                        message_id = id.get(),
                        error = %err,
                        "could not delete an old message"
                    );
                    return;
                }
            }
        }

        // A short page means the channel is now empty.
        if batch < 100 {
            return;
        }
    }
}
