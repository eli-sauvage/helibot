use serenity::{
    all::{
        ChannelId, Context, CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage,
        EditMessage, GetMessages, GuildId, Message, ReactionType, Ready, Timestamp,
    },
    futures::future::join_all,
};
use sqlx::{MySql, Pool};

use crate::{errors::HelibotError, points};
use crate::{points::Point, usernames::UsernameManager};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct MessageBuilder {
    channel_ids: HashMap<GuildId, ChannelId>,
    messages: HashMap<GuildId, (Message, HashSet<Point>)>,
}
impl MessageBuilder {
    pub async fn new(
        ctx: &Context,
        ready_state: &Ready,
        channel_name: &str,
    ) -> Result<MessageBuilder, HelibotError> {
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
        let channels = join_all(guild_channels_future_iter).await;

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

        Ok(MessageBuilder {
            channel_ids,
            messages: HashMap::new(),
        })
    }

    pub async fn print_points_in_all_guilds(
        &mut self,
        ctx: &Context,
        pool: &Pool<MySql>,
        guild_ids: Vec<&GuildId>,
        username_manager: &UsernameManager,
    ) {
        for guild_id in guild_ids {
            let channel_id = match self.channel_ids.get(guild_id) {
                Some(channel_id) => channel_id,
                None => {
                    eprintln!("points channel not found in guild {}", guild_id);
                    continue;
                }
            };

            let points = match points::get_points_for_guild(pool, guild_id).await {
                Ok(points) => points,
                Err(e) => {
                    eprintln!("could not get points for guild {} : {e:?}", e);
                    continue;
                }
            };

            let embed =
                build_point_message(points::parse_to_tuple(username_manager, &points));

            let edit_result = match self.messages.get_mut(guild_id) {
                Some(old_message) => {
                    Some(try_edit_old_points_message(ctx, &embed, &points, old_message).await)
                }
                None => None,
            };
            match edit_result {
                Some(Ok(_)) => {}
                None | Some(Err(_)) => match send_new_msg(ctx, channel_id, embed).await {
                    Ok(new_msg) => {
                        if let Some(old_message_points) = self.messages.get_mut(guild_id) {
                            *old_message_points = (new_msg, points);
                        } else {
                            self.messages.insert(guild_id.to_owned(), (new_msg, points));
                        }
                    }
                    Err(e) => eprintln!(
                        "could not send new msg in channel {} in guild {} : {e:?}",
                        channel_id.get(),
                        guild_id.get()
                    ),
                },
            }
        }
    }
}

async fn send_new_msg(
    ctx: &Context,
    channel_id: &ChannelId,
    embed: CreateEmbed,
) -> Result<Message, HelibotError> {
    delete_old_messages_in_channel(ctx, channel_id).await;
    let buttons = create_buttons();
    let new_message = CreateMessage::new()
        .content("")
        .embed(embed)
        .button(buttons.0)
        .button(buttons.1);

    channel_id
        .send_message(&ctx.http, new_message)
        .await
        .map_err(HelibotError::SerenityError)
}

fn build_point_message(mut points: Vec<(String, String)>) -> CreateEmbed {
    points = points.iter().enumerate().map(|(index, val)|{
        (format!("#{} {}", index + 1, val.0), val.1.to_owned())
    }).collect();
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
    "subalternes: 0, cul-terreux: 500, strapontin: 1500, damoiseau: 5000, cresus: 10000, wakam: 15000, erudit: 20000, abu yaqub: 25000, batracien: 50000, hokage: 75000, bouf royal: 100000"
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
    points: &HashSet<Point>,
    old_message_points: &mut (Message, HashSet<Point>),
) -> Result<(), HelibotError> {
    let (old_message, old_points) = old_message_points;
    if points == old_points {
        println!("points same, skipping");
        return Ok(());
    }
    old_message
        .clone()
        .edit(ctx, EditMessage::new().embed(embed.to_owned()))
        .await
        .map_err(HelibotError::SerenityError)?;
    Ok(())
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

fn create_buttons() -> (CreateButton, CreateButton) {
    let mut refresh_button = CreateButton::new("refresh").label("refresh now");
    if let Ok(refresh_emoji) = ReactionType::try_from("🔄") {
        refresh_button = refresh_button.emoji(refresh_emoji);
    }

    let mut print_all_button = CreateButton::new("print_all").label("diplay all scores");
    if let Ok(scroll_emoji) = ReactionType::try_from("📜") {
        print_all_button = print_all_button.emoji(scroll_emoji);
    }

    (refresh_button, print_all_button)
}
