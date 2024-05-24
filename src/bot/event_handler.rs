use crate::{
    bot::{message::MessageBuilder, points, sessions::ActiveSession, usernames::UsernameManager},
    db_connection::DbConnection,
    Env,
};

use serenity::{
    all::{
        ChannelId, ChannelType, Context, CreateInteractionResponseMessage, EventHandler,
        Interaction, Ready, VoiceState,
    },
    async_trait,
};
use std::time::Duration;
use tokio::time::interval;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("helibot is online");
        let mut client_data = ctx.data.write().await;

        let pool = client_data.get::<DbConnection>().unwrap();
        let username_manager = match UsernameManager::create(&pool, &ctx).await {
            Ok(uname_manager) => uname_manager,
            Err(e) => {
                eprintln!("could not create username manager: {e:?}");
                return;
            }
        };

        let current_sessions_res =
            ActiveSession::add_current_sessions_to_db_on_startup(pool, &ctx, &ready).await;
        if let Err(err) = current_sessions_res {
            println!("could not instanciate active sessions on startup: {err:?}");
        }

        let message_builder = match MessageBuilder::new(
            &ctx,
            &ready,
            &client_data.get::<Env>().unwrap().point_channel_name,
        )
        .await
        {
            Ok(builder) => builder,
            Err(e) => {
                eprintln!("could not create new message builder : {e}");
                return;
            }
        };

        client_data.insert::<UsernameManager>(username_manager);
        client_data.insert::<MessageBuilder>(message_builder);

        let thread_ctx = ctx.clone();
        let thread_client_data = ctx.data.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            let mut client_data = thread_client_data.write().await;
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        client_data.get_mut::<MessageBuilder>().unwrap()
                        .print_points_in_all_guilds(
                            &thread_ctx,
                            ready.guilds.iter().map(|guild| &guild.id).collect(),
                        )
                        .await;
                    }
                }
            }
        });
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        let client_data = ctx.data.read().await;
        let pool = client_data.get::<DbConnection>().unwrap();
        let guild_id = match new_state.guild_id {
            Some(guild_id) => guild_id,
            _ => return,
        };
        let user_id = new_state.user_id;

        if ctx
            .data
            .read()
            .await
            .get::<UsernameManager>()
            .unwrap()
            .get_username_from_cache(guild_id, user_id)
            .is_none()
        {
            if let Some(member) = &new_state.member {
                println!("adding user to db");
                ctx.data
                    .write()
                    .await
                    .get_mut::<UsernameManager>()
                    .unwrap()
                    .add_user(member.clone())
                    .await;
            }
        }

        let voice_state_action =
            VoiceStateAction::compute_action(&ctx, &new_state, old_state.as_ref()).await;

        if voice_state_action == VoiceStateAction::Unchanged {
            return;
        }
        let session = match ActiveSession::get(&pool, user_id.get(), guild_id.get()).await {
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
                        ActiveSession::create(&pool, user_id.get(), guild_id.get()).await
                    {
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

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let mut client_data = ctx.data.write().await;

        if let Interaction::Component(component) = interaction {
            match component.data.custom_id.as_str() {
                "refresh" => {
                    client_data
                        .get_mut::<MessageBuilder>()
                        .unwrap()
                        .print_points_in_all_guilds(&ctx, ctx.cache.guilds().iter().collect())
                        .await;
                    component
                        .create_response(
                            &ctx,
                            serenity::all::CreateInteractionResponse::Acknowledge,
                        )
                        .await
                        .unwrap();
                }
                "print_all" => {
                    //component.user.direct_message(cache_http, builder)
                    if let Some(guild_id) = component.guild_id {
                        if let Ok(table) = points::construct_points_md_table(&ctx, &guild_id).await
                        {
                            component
                                .create_response(
                                    &ctx,
                                    serenity::all::CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content(format!("```md\n{}\n```", table))
                                            .ephemeral(true),
                                    ),
                                )
                                .await
                                .unwrap();
                        }
                    }
                }
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
