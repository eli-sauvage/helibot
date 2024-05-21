use dotenv::dotenv;
use errors::{EnvVarError, HelibotError};
use serenity::{all::GatewayIntents, Client};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::env;

mod errors;
mod event_handler;
mod points;
mod sessions;

struct Env {
    mysql_url: String,
    discord_token: String,
}

#[tokio::main]
async fn main() -> Result<(), errors::HelibotError> {
    let env = retrieve_env().map_err(errors::HelibotError::EnvVarError)?;
    let pool = setup_db_connection(&env).await?;

    let mut client = Client::builder(
        &env.discord_token,
        GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILDS,
    )
    .event_handler(event_handler::Handler { pool })
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
    dotenv().map_err(|_| EnvVarError::DotEnvModuleError)?;
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
    Ok(Env {
        mysql_url,
        discord_token,
    })
}
