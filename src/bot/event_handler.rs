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
        Context, CreateInteractionResponseMessage, EventHandler, GuildId, Interaction, Ready,
        UserId, VoiceState,
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

        for guild_id in &guild_ids {
            if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
                if let Ok(members) = guild.members(&ctx, None, None).await {
                    for member in members {
                        let _ = roles_manager
                            .check_role_for_user(pool, &member.user.id, guild_id, &ctx)
                            .await;
                    }
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
                        println!("tick");
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
        if let Some(member) = &new_state.member {
            ctx.data
                .write()
                .await
                .get_mut::<UsernameManager>()
                .unwrap()
                .add_user_if_not_in_cache(member);

            voice::compute_voice_state_change(&ctx, &new_state, &old_state, &member.guild_id).await;
        };
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let client_data = ctx.data.read().await;

        if let Interaction::Component(component) = interaction {
            match component.data.custom_id.as_str() {
                "refresh" => {
                    println!("refresh");
                    let pool = client_data.get::<DbConnection>().unwrap();
                    let username_manager = client_data.get::<UsernameManager>().unwrap();
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
                        .print_points_in_guild(
                            &ctx,
                            pool,
                            username_manager,
                            role_manager,
                            &guild.id,
                        )
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
