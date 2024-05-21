use serenity::all::{ChannelId, ChannelType, Context, EventHandler, VoiceState};
use serenity::async_trait;
use sqlx::{self, MySql, Pool};

use crate::sessions::ActiveSession;

pub struct Handler {
    pub pool: Pool<MySql>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn voice_state_update(
        &self,
        ctx: Context,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        let guild_id = match new_state.guild_id {
            Some(guild_id) => guild_id,
            _ => return,
        }.get();
        let user_id = new_state.user_id.get();

        let voice_state_action =
            VoiceStateAction::compute_action(&ctx, &new_state, old_state.as_ref()).await;

        if voice_state_action == VoiceStateAction::Unchanged{
            return;
        }
        let session = match ActiveSession::get(&self.pool, user_id, guild_id).await {
            Ok(session) => session,
            Err(e) => {
                eprintln!("{:?}", e);
                return;
            }
        };

        match voice_state_action{
            VoiceStateAction::Joined => {
                if session.is_none(){
                    if let Err(e) = ActiveSession::create(&self.pool, user_id, guild_id).await{
                        eprintln!("{:?}", e);
                    }

                }
            },
            VoiceStateAction::Quit => {
                if let Some(session) = session{
                    if let Err(e) = ActiveSession::terminate(session, &self.pool).await{
                        eprintln!("{:?}", e);
                    }
                }
            },
            VoiceStateAction::Unchanged => {}
            
        }
    }
}

#[derive(Debug, PartialEq)]
enum VoiceStateAction {
    Joined,
    Quit,
    Unchanged,
}
impl VoiceStateAction {
    async fn compute_action(
        ctx: &Context,
        new_voice_state: &VoiceState,
        old_voice_state_opt: Option<&VoiceState>,
    ) -> VoiceStateAction {
        let deaf = new_voice_state.deaf || new_voice_state.self_deaf;
        let old_deaf = old_voice_state_opt
            .is_some_and(|old_voice_state| old_voice_state.deaf || old_voice_state.self_deaf);
        match (old_deaf, deaf) {
            (true, false) => return VoiceStateAction::Joined,
            (false, true) => return VoiceStateAction::Quit,
            _ => {}
        };

        let mute = new_voice_state.mute || new_voice_state.self_mute;
        let old_mute = old_voice_state_opt
            .is_some_and(|old_voice_state| old_voice_state.mute || old_voice_state.self_mute);
        match (old_mute, mute) {
            (true, false) => return VoiceStateAction::Joined,
            (false, true) => return VoiceStateAction::Quit,
            _ => {}
        };

        match (
            is_voice_channel(
                ctx,
                old_voice_state_opt
                    .and_then(|old_state| old_state.channel_id),
            )
            .await,
            is_voice_channel(ctx, new_voice_state.channel_id).await,
        ) {
            (false, true) => return VoiceStateAction::Joined,
            (true, false) => return VoiceStateAction::Quit,
            _ => {}
        };

        VoiceStateAction::Unchanged //default
    }
}

async fn is_voice_channel(ctx: &Context, channel_id_opt: Option<ChannelId>) -> bool {
    if let Some(channel_id) = channel_id_opt {
        if let Ok(channel) = channel_id.to_channel(ctx).await {
            if let Some(guild_channel) = channel.guild() {
                return guild_channel.kind == ChannelType::Voice;
            }
        }
    }
    false
}
