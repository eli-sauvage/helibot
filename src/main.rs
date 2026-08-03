use std::process::ExitCode;
use std::sync::Arc;

use serenity::all::GatewayIntents;
use serenity::Client;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use helibot::config::Config;
use helibot::db::{self, points, sessions};
use helibot::discord::handler::Handler;
use helibot::error::Result;
use helibot::state::AppState;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    match run().await {
        Ok(()) => {
            info!("helibot stopped");
            ExitCode::SUCCESS
        }
        Err(err) => {
            error!(error = %err, "helibot stopped with an error");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let config = Config::load()?;
    let db = db::connect_and_migrate(&config.database_url).await?;

    // Sessions from a previous run are already credited up to their watermark; archive
    // them at that point instead of discarding them.
    let archived = sessions::archive_orphans(&db).await?;
    if archived > 0 {
        info!(archived, "archived sessions left by a previous run");
    }

    let state = Arc::new(AppState::new(db, config));

    // Only non-privileged intents. Requesting a privileged one that is not enabled in
    // the developer portal makes the gateway refuse the connection outright, so member
    // lists are fetched over HTTP instead.
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&state.config.discord_token, intents)
        .event_handler(Handler::new(state.clone()))
        .await?;

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("shutdown requested");
                shard_manager.shutdown_all().await;
            }
            Err(err) => error!(error = %err, "could not listen for shutdown"),
        }
    });

    info!("starting client");
    client.start().await?;

    // Bank whatever the open sessions are worth before exiting, so a planned restart
    // costs nothing at all.
    match db::now(&state.db).await {
        Ok(now) => match points::credit_open_sessions(&state.db, now).await {
            Ok(credited) => info!(credited, "credited open sessions on shutdown"),
            Err(err) => error!(error = %err, "could not credit open sessions on shutdown"),
        },
        Err(err) => error!(error = %err, "could not read the database clock on shutdown"),
    }

    Ok(())
}
