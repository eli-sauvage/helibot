use serenity::all::{ChannelId, ChannelType, Context, GuildId, VoiceState};

use crate::{bot::sessions::ActiveSession, db_connection::DbConnection};

pub async fn compute_voice_state_change(
    ctx: &Context,
    new_state: VoiceState,
    old_state: Option<VoiceState>,
    guild_id: &GuildId,
) {
    let client_data = ctx.data.read().await;
    let pool = client_data.get::<DbConnection>().unwrap();
    let voice_state_action = compute_action(ctx, &new_state, old_state.as_ref()).await;

    if voice_state_action == VoiceStateAction::Unchanged {
        return;
    }
    let session = match ActiveSession::get(pool, new_state.user_id.get(), guild_id.get()).await {
        Ok(session) => session,
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }
    };

    match voice_state_action {
        VoiceStateAction::Joined => {
            if session.is_none() {
                if let Err(e) =
                    ActiveSession::create(pool, new_state.user_id.get(), guild_id.get()).await
                {
                    eprintln!("{:?}", e);
                }
            }
        }
        VoiceStateAction::Quit => {
            if let Some(session) = session {
                if let Err(e) = ActiveSession::terminate(session, pool).await {
                    eprintln!("{:?}", e);
                }
            }
        }
        VoiceStateAction::Unchanged => {}
    }
}

#[derive(Debug, PartialEq)]
enum VoiceStateAction {
    Joined,
    Quit,
    Unchanged,
}

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
            old_voice_state_opt.and_then(|old_state| old_state.channel_id),
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
