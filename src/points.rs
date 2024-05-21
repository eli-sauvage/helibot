use sqlx::{MySql, Pool};

use crate::errors::HelibotError;

#[derive(Debug)]
struct Point {
    id: u32,
    points: u32,
    #[allow(dead_code)]
    guild_id: u64,
    #[allow(dead_code)]
    user_id: u64,
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
