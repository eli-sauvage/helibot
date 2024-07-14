use crate::{
    bot::{
        points::{self, Point},
        roles::RoleManager,
        usernames::UsernameManager,
    },
    db_connection::DbConnection,
    errors::HelibotError,
};

use serenity::{
    all::{
        ChannelId, Context, CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage,
        EditMessage, GetMessages, GuildId, Message, ReactionType, Ready, Timestamp,
    },
    futures::future,
    prelude::TypeMapKey,
};
use sqlx::{MySql, Pool};
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

use super::roles::Seuil;

#[derive(Default)]
pub struct MessagesManager {
    channel_ids: HashMap<GuildId, ChannelId>,
    messages: RwLock<HashMap<GuildId, (Message, HashSet<Point>)>>,
}
impl TypeMapKey for MessagesManager {
    type Value = MessagesManager;
}

impl MessagesManager {
    pub async fn new(
        ctx: &Context,
        ready_state: &Ready,
        channel_name: &str,
    ) -> Result<MessagesManager, HelibotError> {
        let guild_channels_future_iter =
            ready_state
                .guilds
                .clone()
                .into_iter()
                .map(|guild| async move {
                    match guild.id.channels(ctx).await {
                        Ok(channels) => channels
                            .into_iter()
                            .find(|(_, guild_channel)| guild_channel.name == channel_name)
                            .ok_or(HelibotError::PointChannelNotFound(guild.id.get()))
                            .map(|(_, guild_channel)| guild_channel),
                        Err(e) => Err(HelibotError::SerenityError(e)),
                    }
                });
        let channels = future::join_all(guild_channels_future_iter).await;

        //print error (channel not found)
        channels
            .iter()
            .filter(|channel| channel.is_err())
            .for_each(|error| {
                eprintln!(
                    "error while getting points channel in one guild : {:?}",
                    error
                )
            });
        //discard errors and convert to channel ids
        let channel_ids: HashMap<GuildId, ChannelId> = channels
            .into_iter()
            .flatten()
            .map(|guild_channel| (guild_channel.guild_id, guild_channel.id))
            .collect();

        Ok(MessagesManager {
            channel_ids,
            messages: RwLock::new(HashMap::new()),
        })
    }

    pub async fn print_points_in_guild(
        &self,
        ctx: &Context,
        pool: &Pool<MySql>,
        username_manager: &UsernameManager,
        role_manager: &RoleManager,
        guild_id: &GuildId,
    ) {
        let channel_id = match self.channel_ids.get(guild_id) {
            Some(channel_id) => channel_id,
            None => {
                eprintln!("points channel not found in guild {}", guild_id);
                return;
            }
        };

        let points = match points::get_points_for_guild(pool, guild_id).await {
            Ok(points) => points,
            Err(e) => {
                eprintln!("could not get points for guild {} : {e:?}", e);
                return;
            }
        };

        let embed = create_embed(
            points::parse_to_tuple(username_manager, &points),
            role_manager.get_seuils(),
        );
        let mut message_mut = self.messages.write().await;
        let message_guild_mut = message_mut.get_mut(guild_id);
        //message_guild_mut.get_mut(guild_id);
        let edit_success = if let Some((old_message_ref, old_points)) = message_guild_mut {
            println!("bbb1");
            if old_points != &points {
                try_edit_old_points_message(ctx, &embed, old_message_ref)
                    .await
                    .is_ok()
            } else {
                false
            }
        } else {
            false
        };
        if !edit_success {
            match send_new_msg(ctx, channel_id, embed).await {
                Ok(new_msg) => {
                    if let Some(old_message_points) = message_guild_mut {
                        *old_message_points = (new_msg, points);
                    } else {
                        message_mut.insert(guild_id.to_owned(), (new_msg, points));
                    }
                }
                Err(e) => eprintln!(
                    "could not send new msg in channel {} in guild {} : {e:?}",
                    channel_id.get(),
                    guild_id.get()
                ),
            }
        }
        println!("point message created");
    }

    pub async fn print_points_in_all_guilds(&self, ctx: &Context, guild_ids: &Vec<GuildId>) {
        let client_data = ctx.data.read().await;
        let pool = client_data.get::<DbConnection>().unwrap();
        let username_manager = client_data.get::<UsernameManager>().unwrap();
        let role_manager = client_data.get::<RoleManager>().unwrap();
        println!("guild_ids length = {}", guild_ids.len());
        for guild_id in guild_ids {
            self.print_points_in_guild(ctx, pool, username_manager, role_manager, guild_id)
                .await;
        }
    }
}

async fn send_new_msg(
    ctx: &Context,
    channel_id: &ChannelId,
    embed: CreateEmbed,
) -> Result<Message, HelibotError> {
    delete_old_messages_in_channel(ctx, channel_id).await;
    let mut refresh_button = CreateButton::new("refresh").label("refresh");
    if let Ok(refresh_emoji) = ReactionType::try_from("🔄") {
        refresh_button = refresh_button.emoji(refresh_emoji);
    }

    let mut print_all_button = CreateButton::new("print_all").label("afficher tous les scores");
    if let Ok(scroll_emoji) = ReactionType::try_from("📜") {
        print_all_button = print_all_button.emoji(scroll_emoji);
    }
    let new_message = CreateMessage::new()
        .content("")
        .embed(embed)
        .button(refresh_button)
        .button(print_all_button);

    channel_id
        .send_message(&ctx.http, new_message)
        .await
        .map_err(HelibotError::SerenityError)
}

fn create_embed(mut points: Vec<(String, String)>, roles: &[Seuil]) -> CreateEmbed {
    points = points
        .iter()
        .enumerate()
        .map(|(index, val)| (format!("#{} {}", index + 1, val.0), val.1.to_owned()))
        .collect();
    points.shrink_to(15);

    if points.len() > 2 {
        points.insert(2, ("".into(), "".into()));
    }
    if points.len() > 1 {
        points.insert(1, ("".into(), "".into()));
    }
    if !points.is_empty() {
        points.insert(0, ("".into(), "".into()));
    }

    let fields: Vec<(String, String, bool)> = points
        .iter()
        .map(|point| (point.to_owned().0, point.to_owned().1, true))
        .collect();

    let footer = CreateEmbedFooter::new(
        roles
            .iter()
            .map(|r| format!("{} : {}", r.role_name, r.seuil))
            .collect::<Vec<String>>()
            .join(", "),
    );
    CreateEmbed::new()
        .title("Helibot scores. 1 minute en vocal = 1 point")
        .description("Score des 15 premiers. Le message s'update toutes les 3 minutes")
        .footer(footer)
        .fields(fields)
        .timestamp(Timestamp::now())
}

async fn try_edit_old_points_message(
    ctx: &Context,
    embed: &CreateEmbed,
    old_message: &mut Message,
) -> Result<(), HelibotError> {
    old_message
        .edit(ctx, EditMessage::new().embed(embed.to_owned()))
        .await
        .map_err(HelibotError::SerenityError)
}

async fn delete_old_messages_in_channel(ctx: &Context, channel_id: &ChannelId) {
    if let Ok(messages) = channel_id.messages(ctx, GetMessages::new()).await {
        for msg in messages
            .iter()
            .filter(|msg| msg.author.id == ctx.cache.current_user().id)
        {
            let result = msg.delete(ctx).await;
            if let Err(e) = result {
                eprintln!(
                    "could not delete old message in channel {}. Err = {e:?}",
                    channel_id.get()
                )
            }
        }
    }
}
