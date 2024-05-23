use std::time::Duration;

use serenity::all::{
    ChannelId, ChannelType, Context, EventHandler, Interaction,
    Ready, VoiceState,
};
use serenity::async_trait;
use sqlx::{self, MySql, Pool};
use tokio::time::interval;

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::message::MessageBuilder;
use crate::sessions::ActiveSession;
use crate::Env;

pub struct Handler {
    pub pool: Arc<Mutex<Pool<MySql>>>,
    pub message_builder: Arc<RwLock<MessageBuilder>>,
    pub env: Env,
}

#[async_trait]
impl EventHandler for Handler {
    async fn voice_state_update(
        &self,
        ctx: Context,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        let pool = self.pool.lock().await;
        let guild_id = match new_state.guild_id {
            Some(guild_id) => guild_id,
            _ => return,
        }
        .get();
        let user_id = new_state.user_id.get();

        let voice_state_action =
            VoiceStateAction::compute_action(&ctx, &new_state, old_state.as_ref()).await;

        if voice_state_action == VoiceStateAction::Unchanged {
            return;
        }
        let session = match ActiveSession::get(&pool, user_id, guild_id).await {
            Ok(session) => session,
            Err(e) => {
                eprintln!("{:?}", e);
                return;
            }
        };

        match voice_state_action {
            VoiceStateAction::Joined => {
                if session.is_none() {
                    if let Err(e) = ActiveSession::create(&pool, user_id, guild_id).await {
                        eprintln!("{:?}", e);
                    }
                }
            }
            VoiceStateAction::Quit => {
                if let Some(session) = session {
                    if let Err(e) = ActiveSession::terminate(session, &pool).await {
                        eprintln!("{:?}", e);
                    }
                }
            }
            VoiceStateAction::Unchanged => {}
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("helibot is online");

        let pool = self.pool.lock().await;
        *self.message_builder.write().await =
            match MessageBuilder::new(&ctx, &pool, &ready, &__self.env.point_channel_name).await {
                Ok(builder) => builder,
                Err(e) => {
                    eprintln!("could not create new message builder : {e}");
                    return;
                }
            };

        let thread_pool = self.pool.clone();
        let thread_msg_builder = self.message_builder.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60*3));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let pool = thread_pool.lock().await;
                        thread_msg_builder.write().await
                        .print_points_in_all_guilds(
                            &ctx.clone(),
                            &pool,
                            ready.guilds.iter().map(|guild| &guild.id).collect()
                        )
                        .await;
                    }
                }
            }
        });
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Component(component) = interaction {
            match component.data.custom_id.as_str() {
                "refresh" => {
                    let pool = self.pool.lock().await;
                    self.message_builder
                        .write()
                        .await
                        .print_points_in_all_guilds(
                            &ctx.clone(),
                            &pool,
                            ctx.cache.guilds().iter().collect(),
                        )
                        .await;
                    component
                        .create_response(
                            &ctx,
                            serenity::all::CreateInteractionResponse::Acknowledge,
                        )
                        .await
                        .unwrap();
                }
                "print_all" => {}
                _ => println!(
                    "received unknown component custom id on interaction {}",
                    component.data.custom_id
                ),
            }
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
