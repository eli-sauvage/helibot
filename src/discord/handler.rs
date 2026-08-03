use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serenity::all::{
    ChannelId, Context, CreateAttachment, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, EventHandler, Guild,
    GuildId, Interaction, MessageId, Ready, UserId, VoiceState,
};
use serenity::async_trait;
use tracing::{debug, info, warn};

use crate::discord::{leaderboard, roles, usernames, voice};
use crate::state::AppState;
use crate::tasks;

pub struct Handler {
    state: Arc<AppState>,
    background_started: AtomicBool,
}

impl Handler {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            background_started: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!(
            user = %ready.user.name,
            guilds = ready.guilds.len(),
            "connected to discord"
        );
    }

    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        // Fires again after a reconnect; the loops must only be started once.
        if self.background_started.swap(true, Ordering::SeqCst) {
            return;
        }

        info!(
            guilds = guilds.len(),
            "cache ready, starting background work"
        );
        tasks::spawn(self.state.clone(), ctx);
    }

    /// First moment a guild's `voice_states` are actually populated.
    ///
    /// The previous version did its startup scan from `ready`, where the cache is still
    /// empty and the guilds are unavailable stubs, so it never found anyone already in a
    /// voice channel.
    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: Option<bool>) {
        let guild_id = guild.id;
        info!(guild_id = guild_id.get(), name = %guild.name, "guild available");

        let state = self.state.clone();
        tokio::spawn(async move {
            if let Err(err) = voice::reconcile_guild(&state, &ctx, guild_id).await {
                warn!(guild_id = guild_id.get(), error = %err, "could not reconcile voice sessions");
            }
            if let Err(err) = roles::ensure_roles(&state, &ctx, guild_id).await {
                warn!(guild_id = guild_id.get(), error = %err, "could not set up the role ladder");
            }
            if let Err(err) = usernames::sync_guild(&state, &ctx, guild_id).await {
                warn!(guild_id = guild_id.get(), error = %err, "could not sync usernames");
            }
            if let Err(err) = leaderboard::refresh(&state, &ctx, guild_id).await {
                warn!(guild_id = guild_id.get(), error = %err, "could not refresh the board");
            }
        });
    }

    /// Discord invalidates an interaction token after three seconds, so every branch
    /// here answers first and works afterwards. The previous version ran a full refresh
    /// before responding and then called `.unwrap()` on the expired response, which
    /// panicked and left the buttons looking dead.
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Component(component) = interaction else {
            return;
        };
        let Some(guild_id) = component.guild_id else {
            return;
        };

        match component.data.custom_id.as_str() {
            leaderboard::REFRESH_BUTTON => {
                // "Acknowledge" tells Discord an edit to this message is coming.
                if let Err(err) = component
                    .create_response(&ctx, CreateInteractionResponse::Acknowledge)
                    .await
                {
                    warn!(guild_id = guild_id.get(), error = %err, "could not acknowledge refresh");
                    return;
                }

                let state = self.state.clone();
                tokio::spawn(async move {
                    if let Err(err) = leaderboard::refresh(&state, &ctx, guild_id).await {
                        warn!(guild_id = guild_id.get(), error = %err, "manual refresh failed");
                    }
                    if let Err(err) = roles::sync_guild(&state, &ctx, guild_id).await {
                        warn!(guild_id = guild_id.get(), error = %err, "role sweep failed");
                    }
                    if let Err(err) = usernames::sync_guild(&state, &ctx, guild_id).await {
                        warn!(guild_id = guild_id.get(), error = %err, "username sweep failed");
                    }
                });
            }

            leaderboard::SCORES_BUTTON => {
                // Deferred, so the upload does not have to fit in the three seconds.
                let defer = CreateInteractionResponse::Defer(
                    CreateInteractionResponseMessage::new().ephemeral(true),
                );
                if let Err(err) = component.create_response(&ctx, defer).await {
                    warn!(guild_id = guild_id.get(), error = %err, "could not defer scores");
                    return;
                }

                let state = self.state.clone();
                tokio::spawn(async move {
                    let followup = match leaderboard::full_table(&state, guild_id).await {
                        Ok(table) => CreateInteractionResponseFollowup::new()
                            .ephemeral(true)
                            .add_file(CreateAttachment::bytes(table.into_bytes(), "scores.txt")),
                        Err(err) => {
                            warn!(guild_id = guild_id.get(), error = %err, "could not build the scores table");
                            CreateInteractionResponseFollowup::new()
                                .ephemeral(true)
                                .content("Les scores n'ont pas pu être récupérés.")
                        }
                    };

                    if let Err(err) = component.create_followup(&ctx, followup).await {
                        warn!(guild_id = guild_id.get(), error = %err, "could not send the scores");
                    }
                });
            }

            other => debug!(custom_id = other, "unknown component"),
        }
    }

    /// The board is meant to be permanent, so deleting it asks for a new one.
    async fn message_delete(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        guild_id: Option<GuildId>,
    ) {
        let Some(guild_id) = guild_id else {
            return;
        };
        leaderboard::repost_if_deleted(
            &self.state,
            &ctx,
            guild_id,
            channel_id,
            &[deleted_message_id],
        )
        .await;
    }

    async fn message_delete_bulk(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        deleted_message_ids: Vec<MessageId>,
        guild_id: Option<GuildId>,
    ) {
        let Some(guild_id) = guild_id else {
            return;
        };
        leaderboard::repost_if_deleted(
            &self.state,
            &ctx,
            guild_id,
            channel_id,
            &deleted_message_ids,
        )
        .await;
    }

    async fn voice_state_update(&self, ctx: Context, _old: Option<VoiceState>, new: VoiceState) {
        let Some(guild_id) = new.guild_id else {
            warn!("voice state update without a guild id");
            return;
        };

        let closed = match voice::reconcile_guild(&self.state, &ctx, guild_id).await {
            Ok(outcome) => outcome.closed,
            Err(err) => {
                warn!(guild_id = guild_id.get(), error = %err, "could not reconcile voice sessions");
                return;
            }
        };

        // A closed session banks points, which may have pushed the member up a rank.
        for user_id in closed {
            if let Err(err) =
                roles::sync_user(&self.state, &ctx, guild_id, UserId::new(user_id)).await
            {
                warn!(guild_id = guild_id.get(), user_id, error = %err, "could not update roles");
            }
        }
    }
}
