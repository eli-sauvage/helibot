use crate::{bot::sessions::ActiveSession, db_connection::DbConnection, errors::HelibotError};

use serenity::all::{Context, GuildId, UserId};
use sqlx::{types::time::OffsetDateTime, MySql, Pool};
use std::{collections::HashSet, iter::once};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Point {
    pub id: u32,
    pub points: u32,
    pub guild_id: u64,
    pub user_id: u64,
    pub username: String,
    pub score_historique: Option<u32>,
}

pub async fn add_points(
    pool: &Pool<MySql>,
    user_id: u64,
    guild_id: u64,
    points_to_add: i64,
) -> Result<u32, HelibotError> {
    let current_point = match sqlx::query_as!(
        Point,
        "SELECT * FROM Points WHERE user_id = ? AND guild_id = ?",
        user_id,
        guild_id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(id) => id,
        None => {
            sqlx::query!(
                "INSERT INTO Points (user_id, guild_id, points) VALUES (?, ?, ?)",
                user_id,
                guild_id,
                0
            )
            .execute(pool)
            .await?;
            sqlx::query_as!(
                Point,
                "SELECT * FROM Points WHERE user_id = ? AND guild_id = ?",
                user_id,
                guild_id
            )
            .fetch_one(pool)
            .await?
        }
    };
    let new_points = current_point.points + points_to_add as u32;

    sqlx::query!(
        "UPDATE Points SET points=? WHERE id=?",
        new_points,
        current_point.id
    )
    .execute(pool)
    .await?;

    Ok(new_points)
}

pub async fn get_points_for_guild(
    ctx: &Context,
    pool: &Pool<MySql>,
    guild_id: &GuildId,
) -> Result<HashSet<Point>, HelibotError> {
    let mut points = sqlx::query_as!(
        Point,
        "SELECT * from Points WHERE guild_id = ?",
        guild_id.get()
    )
    .fetch_all(pool)
    .await?;

    let active_sessions =
        ActiveSession::get_all_active_sessions_for_guild(pool, guild_id.get()).await?;

    let to_add = if !active_sessions.is_empty() {
        (OffsetDateTime::now_utc() - active_sessions[0].begin).whole_seconds() as u32
    } else {
        0
    };
    for active_session in active_sessions {
        match points
            .iter_mut()
            .find(|point| point.user_id == active_session.user_id)
        {
            Some(point) => {
                point.points += to_add;
            }
            None => {
                println!("manually adding points instance for user {} bc Points row does not exist yet value = {}", active_session.user_id, to_add);
                let user = UserId::new(active_session.user_id).to_user(&ctx).await?;
                let username = user.nick_in(&ctx, guild_id).await.unwrap_or(user.name);
                points.push(Point {
                    id: 0,
                    points: to_add,
                    score_historique: None,
                    guild_id: active_session.guild_id,
                    user_id: active_session.user_id,
                    username,
                })
            }
        }
    }

    Ok(HashSet::from_iter(points.into_iter()))
}

pub async fn get_points_for_user(
    pool: &Pool<MySql>,
    user_id: &UserId,
    guild_id: &GuildId,
) -> Result<Option<Point>, HelibotError> {
    let mut point = sqlx::query_as!(
        Point,
        "SELECT * from Points WHERE guild_id = ? AND user_id = ?",
        guild_id.get(),
        user_id.get()
    )
    .fetch_optional(pool)
    .await?;

    let session = ActiveSession::get(pool, user_id.get(), guild_id.get()).await?;
    if let Some(p) = point.as_mut() {
        if let Some(session) = session {
            p.points += (OffsetDateTime::now_utc() - session.begin).whole_seconds() as u32;
        }
    }
    Ok(point)
}

type Username = String;

pub fn parse_to_tuple(points: &HashSet<Point>) -> Vec<(Username, String)> {
    let mut points: Vec<&Point> = points.iter().collect();
    points.sort_by(|a, b| b.points.cmp(&a.points));
    points
        .iter()
        .map(|point| (point.username.clone(), (point.points / 60).to_string()))
        .collect()
}

const USERNAME_HEADER: &str = "User Name";
const POINTS_HEADER: &str = "Score";

pub async fn construct_points_md_table(
    ctx: &Context,
    guild_id: &GuildId,
) -> Result<String, HelibotError> {
    let client_data = ctx.data.read().await;
    let pool = client_data.get::<DbConnection>().unwrap();
    let points = get_points_for_guild(ctx, pool, guild_id).await?;
    let points_fmt = parse_to_tuple(&points);
    let longest_username_size = points_fmt
        .iter()
        .map(|(username, _)| username.len())
        .chain(once(USERNAME_HEADER.len()))
        .max()
        .unwrap_or(0)
        + 1;
    let longest_points_size = points_fmt
        .iter()
        .map(|(_, points)| points.len())
        .chain(once(POINTS_HEADER.len()))
        .max()
        .unwrap_or(0)
        + 2;
    let mut res = format!(
        "{}{} | {}\n",
        POINTS_HEADER,
        " ".repeat(longest_points_size - POINTS_HEADER.len()),
        USERNAME_HEADER
    );
    res += format!(
        "{}|{}\n",
        "-".repeat(longest_points_size + 1),
        "-".repeat(longest_username_size)
    )
    .as_str();

    res += points_fmt
        .iter()
        .map(|(username, points)| {
            let padding_points = " ".repeat(longest_points_size - points.len());
            format!("{}{} | {}\n", points, padding_points, username,)
        })
        .fold("".to_owned(), |accumulator, current| accumulator + &current)
        .as_str();
    Ok(res)
}
