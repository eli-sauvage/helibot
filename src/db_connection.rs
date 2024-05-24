use crate::{errors::HelibotError, Env};

use serenity::prelude::TypeMapKey;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

pub struct DbConnection;

impl TypeMapKey for DbConnection {
    type Value = Pool<MySql>;
}


pub async fn setup_db_and_migrate(env: &Env) -> Result<Pool<MySql>, HelibotError> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(env.mysql_url.as_str())
        .await?;
    let row: (i64,) = sqlx::query_as("SELECT 150").fetch_one(&pool).await?;
    assert_eq!(row.0, 150); //test connection

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(HelibotError::Migrate)?;

    Ok(pool)
}