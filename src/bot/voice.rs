use serenity::{
    all::{ChannelId, ChannelType, Context, Guild, GuildId, VoiceState},
    prelude::TypeMap,
};

use crate::{db_connection::DbConnection, errors::HelibotError, models::sessions::ActiveSession};

pub trait UpdateVoiceSessions {
    async fn update_sessions(
        &self,
        ctx: &Context,
        client_data: &TypeMap,
        guild: &Guild,
    ) -> Result<(), HelibotError>;
}

impl UpdateVoiceSessions for ChannelId {
    async fn update_sessions(
        &self,
        ctx: &Context,
        client_data: &TypeMap,
        guild: &Guild,
    ) -> Result<(), HelibotError> {
        let pool = client_data.get::<DbConnection>().unwrap();
        let active_user_ids: Vec<u64> = {
            let voice_states_in_channel: Vec<&VoiceState>;
            voice_states_in_channel = guild
                .voice_states
                .iter()
                .filter(|voice| voice.1.channel_id.clone().as_ref() == Some(self))
                .map(|v| v.1)
                .collect();
            voice_states_in_channel
                .iter()
                .filter(|voice| {
                    !(voice.member.as_ref().is_some_and(|mem| mem.user.bot)
                        || voice.deaf
                        || voice.self_deaf
                        || voice.mute
                        || voice.self_mute)
                })
                .map(|voice| voice.user_id.get())
                .collect()
        };
        let mut active_sessions =
            ActiveSession::get_all_active_sessions_for_guild(pool, guild.id.get()).await?;

        let user_ids_with_active_session: Vec<u64> =
            active_sessions.iter().map(|s| s.user_id).collect();
        if active_user_ids.len() == 1 {
            if let Some(session_index) = active_sessions
                .iter()
                .position(|s| s.user_id == active_user_ids[0])
            {
                let session = active_sessions.remove(session_index);
                ActiveSession::terminate(session, ctx, pool).await?;
            }
        } else {
            for active_user_id_without_session in active_user_ids
                .iter()
                .filter(|user_id| !user_ids_with_active_session.contains(user_id))
            {
                ActiveSession::create(pool, *active_user_id_without_session, guild.id.get())
                    .await?;
            }
        }
        println!("active sessions in db : {:?}", active_sessions);
        println!("active uid in discord : {:?}", user_ids_with_active_session);
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
            println!("DELETING");
            session.terminate(ctx, pool).await?;
        }
        Ok(())
    }
}

pub trait UpdateSessionsForAllChannels {
    async fn update_sessions_for_all_channels(&self, ctx: &Context, state: &TypeMap);
}
impl UpdateSessionsForAllChannels for Guild {
    async fn update_sessions_for_all_channels(&self, ctx: &Context, state: &TypeMap) {
        let channels = self.channels(ctx).await;
        let voice_channels: Vec<_> = channels
            .iter()
            .flat_map(|channels| {
                channels
                    .iter()
                    .map(|(_, channel)| channel)
                    .filter(|channel| channel.kind == ChannelType::Voice)
            })
            .collect();
        // let g = Arc::new(ctx.cache.clone().guild(self).unwrap().clone());
        for voice_channel in voice_channels {
            let res = voice_channel
                .id
                .update_sessions(ctx, state, &self)
                .await;
            if let Err(e) = res {
                eprintln!(
                    "could not update voice session on bot startup, guild = {} channel = {}: {e:?}",
                    self.id, voice_channel.id
                );
            }
        }
    }
}
