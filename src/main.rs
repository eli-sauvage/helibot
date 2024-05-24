mod bot;
mod db_connection;
mod errors;
use bot::{event_handler, sessions};
use db_connection::DbConnection;
use errors::{EnvVarError, HelibotError};

use serenity::{all::GatewayIntents, prelude::TypeMapKey, Client};
use std::env;

#[tokio::main]
async fn main() -> Result<(), errors::HelibotError> {
    let env = get_env().map_err(HelibotError::EnvVarError)?;
    let pool = db_connection::setup_db_and_migrate(&env).await?;

    sessions::detect_and_remove_dangling_sessions(&pool).await?;

    let mut client = Client::builder(
        &env.discord_token,
        GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILDS,
    )
    .event_handler(event_handler::Handler)
    .await
    .map_err(HelibotError::SerenityError)?;

    let mut client_data = client.data.write().await;
    client_data.insert::<DbConnection>(pool);
    client_data.insert::<Env>(env);
    drop(client_data);

    if let Err(err) = client.start().await {
        println!("Client error: {err:?}");
    }

    //current thread blocked until client error
    //TODO: use never type (!) when in stable rust
    Ok(())
}

struct Env {
    mysql_url: String,
    discord_token: String,
    point_channel_name: String,
}
impl TypeMapKey for Env {
    type Value = Env;
}

fn get_env() -> Result<Env, errors::EnvVarError> {
    dotenvy::dotenv().map_err(EnvVarError::DotEnvModuleError)?;
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
