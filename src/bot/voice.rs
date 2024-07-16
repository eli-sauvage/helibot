use std::sync::Arc;

use serenity::{
    all::{ChannelId, Context, Guild, VoiceState},
    prelude::TypeMap,
};

use crate::{bot::sessions::ActiveSession, db_connection::DbConnection, errors::HelibotError};

pub async fn update_voice_sessions(
    ctx: &Context,
    client_data: &TypeMap,
    channel: &ChannelId,
    guild: Arc<Guild>,
) -> Result<(), HelibotError> {
    let pool = client_data.get::<DbConnection>().unwrap();
    // let guild = guild.read().await;
    let voice_states_in_channel: Vec<&VoiceState> = guild
        .voice_states
        .iter()
        .filter(|voice| voice.1.channel_id.as_ref() == Some(channel))
        .map(|v| v.1)
        .collect();
    let active_user_ids: Vec<u64> = voice_states_in_channel
        .iter()
        .filter(|voice| {
            !(voice.member.as_ref().is_some_and(|mem| mem.user.bot)
                || voice.deaf
                || voice.self_deaf
                || voice.mute
                || voice.self_mute)
        })
        .map(|voice| voice.user_id.get())
        .collect();
    let mut active_sessions =
        ActiveSession::get_all_active_sessions_for_guild(pool, guild.id.get()).await?;

    let user_ids_with_active_session: Vec<u64> =
        active_sessions.iter().map(|s| s.user_id).collect();
    if active_user_ids.len() == 1 {
        if let Some(session_index) = active_sessions
            .iter()
            .position(|s| s.user_id == active_user_ids[0])
        {
            println!("alone session found");
            let session = active_sessions.remove(session_index);
            ActiveSession::terminate(session, ctx, pool).await?;
        }
    } else {
        for active_user_id_without_session in active_user_ids
            .iter()
            .filter(|user_id| !user_ids_with_active_session.contains(user_id))
        {
            ActiveSession::create(pool, *active_user_id_without_session, guild.id.get()).await?;
        }
    }
    let sessions_index_to_terminate: Vec<_> = active_sessions
        .iter()
        .enumerate()
        .filter(|(_, session)| {
            user_ids_with_active_session.contains(&session.user_id)
                && !active_user_ids.contains(&session.user_id)
        })
        .map(|(index, _)| index)
        .rev() // reverse to start deleting from the end of the list
        .collect();

    for session_index_to_terminate in sessions_index_to_terminate {
        let session = active_sessions.remove(session_index_to_terminate);
        session.terminate(ctx, pool).await?;
    }
    Ok(())
}
