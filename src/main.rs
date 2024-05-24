mod bot;
mod errors;
use bot::{event_handler, sessions};
use errors::{EnvVarError, HelibotError};


use dotenvy::dotenv;
use serenity::{all::GatewayIntents, Client};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use tokio::sync::RwLock;
use std::{env, sync::Arc};


struct Env {
    mysql_url: String,
    discord_token: String,
    point_channel_name: String,
}

#[tokio::main]
async fn main() -> Result<(), errors::HelibotError> {
    let env = retrieve_env().map_err(HelibotError::EnvVarError)?;
    let pool = setup_db_connection(&env).await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(HelibotError::Migrate)?;

    sessions::ActiveSession::detect_and_remove_dangling_sessions(&pool).await?;

    let mut client = Client::builder(
        &env.discord_token,
        GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILDS,
    )
    .event_handler(event_handler::Handler {
        pool: Arc::new(RwLock::new(pool)),
        env,
        username_manager: Default::default(),
        message_builder: Default::default(),
    })
    .await
    .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }

    Ok(())
}

async fn setup_db_connection(env: &Env) -> Result<Pool<MySql>, HelibotError> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(env.mysql_url.as_str())
        .await?;
    let row: (i64,) = sqlx::query_as("SELECT 150").fetch_one(&pool).await?;
    assert_eq!(row.0, 150); //test connection
    Ok(pool)
}

fn retrieve_env() -> Result<Env, errors::EnvVarError> {
    dotenv().map_err(EnvVarError::DotEnvModuleError)?;
    fn get_var(var: &str) -> Result<String, errors::EnvVarError> {
        env::var_os(var)
            .ok_or(EnvVarError::VarNotFound)?
            .into_string()
            .map_err(EnvVarError::OsString)
    }
    let mysql_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        get_var("MARIADB_USER")?,
        get_var("MARIADB_PASSWORD")?,
        get_var("MARIADB_HOST")?,
        get_var("MARIADB_PORT")?,
        get_var("MARIADB_DATABASE")?
    );

    let discord_token = get_var("DISCORD_TOKEN")?;

    let point_channel_name = get_var("POINTS_CHANNEL_NAME")?;

    Ok(Env {
        mysql_url,
        discord_token,
        point_channel_name,
    })
}
