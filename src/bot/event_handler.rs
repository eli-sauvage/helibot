use crate::{
    bot::{
        message::MessageBuilder, points, sessions::ActiveSession, usernames::UsernameManager, voice,
    },
    db_connection::DbConnection,
    Env,
};

use serenity::{
    all::{
        Context, CreateInteractionResponseMessage, EventHandler, Interaction, Ready, VoiceState,
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

        let message_builder = match MessageBuilder::new(
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

        client_data.insert::<UsernameManager>(username_manager);
        client_data.insert::<MessageBuilder>(message_builder);
        drop(client_data);

        let thread_client_data = ctx.data.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            let mut client_data = thread_client_data.write().await;
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        client_data.get_mut::<MessageBuilder>().unwrap()
                        .print_points_in_all_guilds(
                            &ctx,
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
