use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use serenity::all::Context;
use tokio::time::{interval, MissedTickBehavior};
use tracing::{debug, error, warn};

use crate::db::{self, points};
use crate::discord::{leaderboard, roles, usernames};
use crate::state::AppState;

/// How long to wait before restarting a background task that died.
const RESTART_DELAY: Duration = Duration::from_secs(5);

pub fn spawn(state: Arc<AppState>, ctx: Context) {
    let credit_state = state.clone();
    supervise("credit", move || credit_loop(credit_state.clone()));

    let refresh_state = state.clone();
    let refresh_ctx = ctx.clone();
    supervise("refresh", move || {
        refresh_loop(refresh_state.clone(), refresh_ctx.clone())
    });

    supervise("usernames", move || {
        usernames_loop(state.clone(), ctx.clone())
    });
}

/// Keeps a background loop alive for the life of the process.
///
/// A panic inside a detached `tokio::spawn` kills only that task, and the client happily
/// carries on without it. That is how the previous version lost its refresh loop: one
/// transient database error hit a `panic!`, and the board silently stopped updating for
/// months while the process stayed up.
fn supervise<F, Fut>(name: &'static str, task: F)
where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        loop {
            match tokio::spawn(task()).await {
                Ok(()) => warn!(task = name, "background task returned, restarting"),
                Err(err) => {
                    error!(task = name, error = %err, "background task panicked, restarting")
                }
            }
            tokio::time::sleep(RESTART_DELAY).await;
        }
    });
}

fn ticker(period_secs: u64, floor_secs: u64) -> tokio::time::Interval {
    let mut ticker = interval(Duration::from_secs(period_secs.max(floor_secs)));
    // Falling behind should not produce a burst of catch-up ticks.
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    ticker
}

/// Banks the voice time of every open session. This is what bounds the loss from an
/// unclean shutdown to one interval.
async fn credit_loop(state: Arc<AppState>) {
    let mut ticker = ticker(state.config.credit_interval_secs, 5);

    loop {
        ticker.tick().await;

        let now = match db::now(&state.db).await {
            Ok(now) => now,
            Err(err) => {
                error!(error = %err, "could not read the database clock");
                continue;
            }
        };

        match points::credit_open_sessions(&state.db, now).await {
            Ok(0) => {}
            Ok(credited) => debug!(credited, "credited open sessions"),
            Err(err) => error!(error = %err, "could not credit open sessions"),
        }
    }
}

async fn refresh_loop(state: Arc<AppState>, ctx: Context) {
    let mut ticker = ticker(state.config.refresh_interval_secs, 10);

    loop {
        ticker.tick().await;

        for guild_id in ctx.cache.guilds() {
            if let Err(err) = leaderboard::refresh(&state, &ctx, guild_id).await {
                error!(guild_id = guild_id.get(), error = %err, "could not refresh the board");
            }
            if let Err(err) = roles::sync_guild(&state, &ctx, guild_id).await {
                error!(guild_id = guild_id.get(), error = %err, "could not sync roles");
            }
        }
    }
}

async fn usernames_loop(state: Arc<AppState>, ctx: Context) {
    let mut ticker = ticker(state.config.username_refresh_interval_secs, 60);

    loop {
        ticker.tick().await;

        for guild_id in ctx.cache.guilds() {
            if let Err(err) = usernames::sync_guild(&state, &ctx, guild_id).await {
                error!(guild_id = guild_id.get(), error = %err, "could not sync usernames");
            }
        }
    }
}
