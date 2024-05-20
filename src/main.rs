use dotenv::dotenv;
use errors::EnvVarError;
use serenity::{all::GatewayIntents, Client};
use sqlx::mysql::MySqlPoolOptions;
use std::env;

mod errors;
mod event_handler;
mod sessions;
mod points;

struct Env {
    mysql_url: String,
    discord_token: String,
}

#[tokio::main]
async fn main() -> Result<(), errors::HelibotError> {
    let env = construct_msql_url().map_err(errors::HelibotError::EnvVarError)?;
    println!("{}", env.mysql_url);
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(env.mysql_url.as_str())
        .await?;
    let row: (i64,) = sqlx::query_as("SELECT 150").fetch_one(&pool).await?;
    assert_eq!(row.0, 150); //test connection

    let intents = GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILDS;
    let handler = event_handler::Handler { pool };

    let mut client = Client::builder(&env.discord_token, intents)
        .event_handler(handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }

    println!("bot has joined");

    Ok(())
}

fn construct_msql_url() -> Result<Env, errors::EnvVarError> {
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
