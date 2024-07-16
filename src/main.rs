mod bot;
mod conf;
mod db_connection;
mod errors;

use bot::{event_handler, sessions};
use conf::Env;
use db_connection::DbConnection;
use errors::HelibotError;

use serenity::{all::GatewayIntents, Client};

//#[tokio::main]
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), errors::HelibotError> {

    let env = Env::get_env()?;
    let pool = db_connection::setup_db_and_migrate(&env).await?;

    sessions::detect_and_remove_dangling_sessions(&pool).await?;

    let mut client = Client::builder(
        env.get_discord_token(),
        GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILDS,
    )
    .event_handler(event_handler::Handler)
    .await
    .map_err(HelibotError::SerenityError)?;

    let mut client_data = client.data.write().await;
    client_data.insert::<DbConnection>(pool);
    client_data.insert::<Env>(env);
    drop(client_data);

    println!("starting client");

    if let Err(err) = client.start().await {
        println!("Client error: {err:?}");
    }

    //current thread blocked until client error
    //TODO: use never type (!) when in stable rust
    Ok(())
}
