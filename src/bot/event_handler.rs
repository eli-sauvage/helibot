use crate::{
    bot::voice::{self, UpdateSessionsForAllChannels},
    db_connection::DbConnection,
    managers::{message::MessagesManager, roles::RoleManager, usernames::UsernameManager},
    models::{points, sessions::ActiveSession},
    Env,
};

use serenity::{
    all::{
        ChannelType, Context, CreateAttachment, CreateInteractionResponseMessage, EventHandler, Guild, GuildId, Interaction, Ready, UserId, VoiceState
    },
    async_trait,
};
use std::{sync::Arc, time::Duration};
use tokio::time::interval;

use super::voice::UpdateVoiceSessions;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("helibot is online");
        let state = ctx.data.read().await;

        let pool = state.get::<DbConnection>().unwrap();
        let guild_ids: Vec<GuildId> = ready.guilds.iter().map(|guild| guild.id).collect();

        print!("adding current sessions to db ...");
        ActiveSession::add_current_sessions_to_db_on_startup(pool, &ctx, &ready).await;
        println!("done\n");

        println!("instanciating messages manager ...");
        let messages_manager = MessagesManager::new(
            &ctx,
            &ready,
            &state.get::<Env>().unwrap().point_channel_name,
        )
        .await;
        println!("done\n");

        println!("instanciating roles manager...");
        let roles_manager =
            RoleManager::new(&ctx, &guild_ids, state.get::<Env>().unwrap().roles.clone()).await;

        let username_manager = UsernameManager::new(&guild_ids);

        drop(state);
        let mut state_mut = ctx.data.write().await;
        state_mut.insert::<UsernameManager>(username_manager);
        state_mut.insert::<MessagesManager>(messages_manager);
        state_mut.insert::<RoleManager>(roles_manager);
        drop(state_mut);

        for guild_id in guild_ids.clone() {
        }

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
                                panic!("could not get active sessions on refresh : {e:?}")
                            },
                        };
                        for session in active_sessions{
                            if let Err(e)  = client_data.get::<RoleManager>().unwrap()
                                .check_role_for_single_user(&ctx, pool,&UserId::new(session.user_id),&GuildId::new(session.guild_id))
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

    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: Option<bool>){
            let ctx_thread = ctx.clone();
            tokio::spawn(async move {
                let state = ctx_thread.data.read().await;
                guild
                    .clone()
                    .update_sessions_for_all_channels(&ctx_thread, &state)
                    .await;
            });

            let ctx_thread = ctx.clone();
            tokio::spawn(async move {
                let state = ctx_thread.data.read().await;
                let roles_manager = state.get::<RoleManager>().unwrap();
                // let _ = roles_manager
                //     .check_role_for_every_user_in_guild(&ctx_thread, &guild_id)
                //     .await;
            });
            let ctx_thread = ctx.clone();
            tokio::spawn(async move {
                let state = ctx_thread.data.read().await;
                let username_manager = state.get::<UsernameManager>().unwrap();
                // if let Err(e) = username_manager
                //     .refresh_usernames(&ctx_thread, &guild_id)
                //     .await
                // {
                //     eprintln!("could not refresh uname : {e:?}")
                // }
            });

    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        let client_data = ctx.data.read().await;
        let guild_id = match new_state.guild_id {
            Some(guild_id) => guild_id,
            None => {
                eprintln!("received voicestate without guild_id");
                return;
            }
        };
        if let Some(old_channel) = &old_state.as_ref().and_then(|os| os.channel_id) {
            let r = old_channel
                .update_sessions(&ctx, &client_data, &guild_id)
                .await;
            // let r = voice::update_voice_sessions(&ctx, &client_data, old_channel, g.clone()).await;
            if let Err(e) = r {
                eprintln!(
                    "could not update channel {} in guild {:?} : {e:?}",
                    old_channel,
                    &old_state.map(|os| os.guild_id)
                )
            }
        }
        if let Some(new_channel) = new_state.channel_id {
            let r = new_channel
                .update_sessions(&ctx, &client_data, &guild_id)
                .await;
            if let Err(e) = r {
                eprintln!(
                    "could not update channel {} in guild {:?} : {e:?}",
                    new_channel, &new_state.guild_id
                )
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let client_data = ctx.data.read().await;

        if let Interaction::Component(component) = interaction {
            match component.data.custom_id.as_str() {
                "refresh" => {
                    let guild_id = match component.guild_id {
                        Some(gid) => gid,
                        None => return,
                    };
                    if let Err(e) = client_data
                        .get::<MessagesManager>()
                        .unwrap()
                        .print_points_in_guild(&ctx, &client_data, &guild_id)
                        .await
                    {
                        eprintln!(
                            "could not update error msg in {}<{}> : {e:?}",
                            guild_id.name(&ctx).unwrap_or("undef".into()),
                            guild_id.get()
                        );
                    };
                    component
                        .create_response(
                            &ctx,
                            serenity::all::CreateInteractionResponse::Acknowledge,
                        )
                        .await
                        .unwrap();

                    let ctx_thread = ctx.clone();
                    tokio::spawn(async move {
                        let c_data = ctx_thread.data.read().await;
                        let role_manager = c_data.get::<RoleManager>().unwrap();
                        if let Err(e) = role_manager
                            .check_role_for_every_user_in_guild(&ctx_thread, &guild_id)
                            .await
                        {
                            eprintln!(
                                "could not check roles in guild {} on manual refresh: {e:?}",
                                guild_id
                            )
                        }
                    });
                    let thread_ctx = ctx.clone();
                    tokio::spawn(async move {
                        let c_data = thread_ctx.data.read().await;
                        let username_manager = c_data.get::<UsernameManager>().unwrap();
                        if let Err(e) = username_manager
                            .refresh_usernames(&thread_ctx, &guild_id)
                            .await
                        {
                            eprintln!("could not refresh uname : {e:?}")
                        }
                    });
                }
                "print_all" => {
                    if let Some(guild_id) = component.guild_id {
                        if let Ok(table) = points::construct_points_md_table(&ctx, &guild_id).await
                        {
                            let bytes: Vec<_> = table.bytes().collect();
                            let attachment = CreateAttachment::bytes(bytes, "scores.txt");
                            component
                                .create_response(
                                    &ctx,
                                    serenity::all::CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            //.content(format!("```md\n{}\n```", table))
                                            .ephemeral(true)
                                            .add_file(attachment),
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
