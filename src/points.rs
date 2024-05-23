use std::collections::HashSet;

use serenity::all::{GuildId, UserId};
use sqlx::{types::time::OffsetDateTime, MySql, Pool};

use crate::{errors::HelibotError, sessions::ActiveSession, usernames::UsernameManager};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub id: u32,
    pub points: u32,
    pub guild_id: u64,
    pub user_id: u64,
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

    active_sessions.iter().for_each(|active_session| {
        if let Some(point) = points
            .iter_mut()
            .find(|point| point.user_id == active_session.user_id)
        {
            let to_add = (OffsetDateTime::now_utc() - active_sessions[0].begin).whole_seconds();
            point.points += to_add as u32
        }
    });

    Ok(HashSet::from_iter(points.into_iter()))
}

type Username = String;

pub fn parse_to_tuple(
    username_manager: &UsernameManager,
    points: &HashSet<Point>,
) -> Vec<(Username, String)> {
    let mut points: Vec<&Point> = points.iter().collect();
    points.sort_by(|a, b| b.points.cmp(&a.points));
    points
        .iter()
        .filter_map(|point| {
            username_manager
                .get_username_from_cache(GuildId::new(point.guild_id), UserId::new(point.user_id))
                .map(|username| (username.to_string(), point.points.to_string()))
        })
        .collect()
}

pub async fn construct_points_md_table(
    pool: &Pool<MySql>,
    guild_id: &GuildId,
    username_manager: &UsernameManager,
) -> Result<String, HelibotError> {
    let points = get_points_for_guild(pool, guild_id).await?;
    let mut points_fmt = parse_to_tuple(username_manager, &points);
    points_fmt.insert(0, ("User Name".into(), "Score".into()));
    let longest_username_size = points_fmt
        .iter()
        .map(|(username, _)| username.len())
        .max()
        .unwrap_or(0)
        + 2;
    let longest_points_size = points_fmt
        .iter()
        .map(|(_, points)| points.len())
        .max()
        .unwrap_or(0)
        + 2;
    points_fmt.insert(
        1,
        (
            "-".repeat(longest_username_size - 2),
            "-".repeat(longest_points_size - 2),
        ),
    );

    Ok(points_fmt
        .iter()
        .map(|(username, points)| {
            format!(
                "|{:username_width$}|{:points_width$}|\n",
                format!(" {} ", username),
                format!(" {} ", points),
                username_width = longest_username_size,
                points_width = longest_points_size
            )
        })
        .fold("".to_owned(), |accumulator, current| accumulator + &current))
}
