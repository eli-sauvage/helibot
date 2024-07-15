use crate::{
    bot::{
        message::MessagesManager, points, roles::RoleManager, sessions::ActiveSession,
        usernames::UsernameManager, voice,
    },
    db_connection::DbConnection,
    Env,
};

use serenity::{
    all::{
        ChannelType, Context, CreateInteractionResponseMessage, EventHandler, GuildId, Interaction,
        Ready, UserId, VoiceState,
    },
    async_trait,
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::interval};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("helibot is online");
        let mut client_data = ctx.data.write().await;

        let pool = client_data.get::<DbConnection>().unwrap();
        let guild_ids: Vec<GuildId> = ready.guilds.iter().map(|guild| guild.id).collect();

        let username_manager = match UsernameManager::create(pool, &ctx).await {
            Ok(uname_manager) => uname_manager,
            Err(e) => {
                panic!("could not create username manager: {e:?}");
            }
        };

        let current_sessions_res =
            ActiveSession::add_current_sessions_to_db_on_startup(pool, &ctx, &ready).await;
        if let Err(err) = current_sessions_res {
            println!("could not instanciate active sessions on startup: {err:?}");
        }

        let message_builder = match MessagesManager::new(
            &ctx,
            &ready,
            &client_data.get::<Env>().unwrap().point_channel_name,
        )
        .await
        {
            Ok(builder) => builder,
            Err(e) => {
                panic!("could not create new message builder : {e}");
            }
        };

        let roles_manager = match RoleManager::new(
            &ctx,
            &guild_ids,
            client_data.get::<Env>().unwrap().roles.clone(),
        )
        .await
        {
            Ok(rm) => rm,
            Err(e) => {
                panic!("could not instantiate role manager : {e:?}");
            }
        };

        println!(
            "helibot present in guilds {}",
            guild_ids
                .iter()
                .filter_map(|gid| gid.name(&ctx))
                .collect::<Vec<_>>()
                .join(", "),
        );

        for guild_id in &guild_ids {
            let guild = ctx.cache.clone().guild(guild_id).unwrap().clone();
            if let Ok(members) = guild.members(&ctx, None, None).await {
                for member in members {
                    let _ = roles_manager
                        .check_role_for_user(pool, &member.user.id, guild_id, &ctx)
                        .await;
                }
            }
            let channels = guild.channels(&ctx).await;
            let voice_channels: Vec<_> = channels
                .iter()
                .flat_map(|channels| {
                    channels
                        .iter()
                        .map(|(_, channel)| channel)
                        .filter(|channel| channel.kind == ChannelType::Voice)
                })
                .collect();
            let g = Arc::new(RwLock::new(
                ctx.cache.clone().guild(guild_id).unwrap().clone(),
            ));
            for voice_channel in voice_channels {
                let r =
                    voice::update_voice_sessions(&client_data, &voice_channel.id, g.clone()).await;
                if let Err(e) = r {
                    eprintln!("could not update voice session on bot startup, guild = {} channel = {}: {e:?}", guild_id, voice_channel.id);
                }
            }
        }
        client_data.insert::<UsernameManager>(username_manager);
        client_data.insert::<MessagesManager>(message_builder);
        client_data.insert::<RoleManager>(roles_manager);
        drop(client_data);

        let thread_client_data = ctx.data.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3 * 60));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let client_data = thread_client_data.read().await;
                        client_data.get::<MessagesManager>().unwrap().print_points_in_all_guilds(
                            &ctx,
                            &guild_ids,
                        )
                        .await;
                        let pool = client_data.get::<DbConnection>().unwrap();
                        let active_sessions = match ActiveSession::get_all_active_sessions(pool).await {
                            Ok(act_s) => act_s,
                            Err(e) => {
                                panic!("could not connect to db on refresh : {e:?}")
                            },
                        };
                        for session in active_sessions{
                            if let Err(e)  = client_data.get::<RoleManager>().unwrap()
                                .check_role_for_user(pool,&UserId::new(session.user_id),&GuildId::new(session.guild_id), &ctx)
                                .await
                            {
                                eprintln!("could not update role for user {} in guild {} : {e:?}", session.user_id, session.guild_id);
                            }
                        }
                    }
                }
            }
        });
        println!("ready");
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        let g = Arc::new(RwLock::new(
            ctx.cache
                .clone()
                .guild(new_state.guild_id.unwrap())
                .unwrap()
                .clone(),
        ));
        let mut client_data = ctx.data.write().await;
        if let Some(member) = &new_state.member {
            client_data
                .get_mut::<UsernameManager>()
                .unwrap()
                .add_user_if_not_in_cache(member);
            if let Some(old_channel) = &old_state.as_ref().and_then(|os| os.channel_id) {
                let r = voice::update_voice_sessions(&client_data, old_channel, g.clone()).await;
                if let Err(e) = r {
                    eprintln!(
                        "could not update channel {} in guild {:?} : {e:?}",
                        old_channel,
                        &old_state.map(|os| os.guild_id)
                    )
                }
            }
            if let Some(new_channel) = new_state.channel_id {
                let r = voice::update_voice_sessions(&client_data, &new_channel, g.clone()).await;
                if let Err(e) = r {
                    eprintln!(
                        "could not update channel {} in guild {:?} : {e:?}",
                        new_channel, &new_state.guild_id
                    )
                }
            }
        };
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let client_data = ctx.data.read().await;

        if let Interaction::Component(component) = interaction {
            match component.data.custom_id.as_str() {
                "refresh" => {
                    let pool = client_data.get::<DbConnection>().unwrap();
                    let role_manager = client_data.get::<RoleManager>().unwrap();
                    let guild = match component.guild_id.map(|g_id| g_id.to_partial_guild(&ctx)) {
                        Some(g_id) => g_id.await,
                        None => return,
                    };
                    let guild = match guild {
                        Ok(gid) => gid,
                        Err(_) => return,
                    };
                    client_data
                        .get::<MessagesManager>()
                        .unwrap()
                        .print_points_in_guild(&ctx, &client_data, &guild.id)
                        .await;
                    component
                        .create_response(
                            &ctx,
                            serenity::all::CreateInteractionResponse::Acknowledge,
                        )
                        .await
                        .unwrap();

                    let guild_members = match guild.members(&ctx, None, None).await {
                        Ok(gm) => gm,
                        Err(e) => {
                            eprintln!(
                                "could not fetch guild members for guild {} : {e:?}",
                                guild.id
                            );
                            return;
                        }
                    };
                    for member in guild_members {
                        if let Err(e) = role_manager
                            .check_role_for_user(pool, &member.user.id, &guild.id, &ctx)
                            .await
                        {
                            eprintln!(
                                "could not update role for user {} in guild {} : {e:?}",
                                member.user.id, guild.id
                            );
                        };
                    }
                }
                "print_all" => {
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
