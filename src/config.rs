use serde::Deserialize;
use tracing::{debug, warn};

use crate::error::Result;

/// One rank in the ladder. `seuil` is a threshold in *minutes* of voice time, while
/// points are stored in seconds, which is why the comparison goes through
/// [`RoleTier::threshold_seconds`] rather than comparing the raw numbers.
#[derive(Debug, Clone, Deserialize)]
pub struct RoleTier {
    pub seuil: u32,
    pub role_name: String,
}

impl RoleTier {
    pub fn threshold_seconds(&self) -> u64 {
        u64::from(self.seuil) * 60
    }
}

/// Per-guild overrides. Resolving the points channel by id avoids the previous
/// behaviour of looking it up by name, which silently broke whenever the channel was
/// renamed.
#[derive(Debug, Clone, Deserialize)]
pub struct GuildConfig {
    pub id: u64,
    pub points_channel_id: Option<u64>,
}

fn default_refresh_interval() -> u64 {
    180
}

fn default_credit_interval() -> u64 {
    60
}

fn default_username_interval() -> u64 {
    3600
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub discord_token: String,

    /// Fallback for guilds with no `points_channel_id`. Kept so an existing
    /// `helibot.toml` keeps working untouched.
    pub point_channel_name: String,

    #[serde(default)]
    pub guilds: Vec<GuildConfig>,

    /// How often the leaderboard message and the role ladder are refreshed.
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,

    /// How often open sessions are credited to `Points`. Also the upper bound on how
    /// much voice time an unclean shutdown can lose.
    #[serde(default = "default_credit_interval")]
    pub credit_interval_secs: u64,

    /// How often stored usernames are reconciled with Discord. Slower than the
    /// leaderboard refresh because it needs a full member list per guild.
    #[serde(default = "default_username_interval")]
    pub username_refresh_interval_secs: u64,

    pub roles: Vec<RoleTier>,
}

impl Config {
    /// Reads `helibot.toml`, then overlays `HELIBOT_`-prefixed environment variables
    /// (`HELIBOT_DATABASE_URL`, `HELIBOT_DISCORD_TOKEN`, ...).
    pub fn load() -> Result<Self> {
        // A missing .env is normal in Docker, where the values are already in the
        // environment. The previous version treated it as fatal.
        match dotenvy::dotenv() {
            Ok(path) => debug!(path = %path.display(), "loaded .env"),
            Err(err) if err.not_found() => debug!("no .env file, using the environment"),
            Err(err) => warn!(error = %err, "could not load .env, using the environment"),
        }

        let mut config: Config = config::Config::builder()
            .add_source(config::File::with_name("helibot"))
            .add_source(config::Environment::with_prefix("HELIBOT"))
            .build()?
            .try_deserialize()?;

        config.roles.sort_by_key(|tier| tier.seuil);

        if config.roles.is_empty() {
            warn!("no roles configured, the role ladder is disabled");
        } else if config.roles[0].seuil != 0 {
            warn!(
                lowest = config.roles[0].seuil,
                "lowest role threshold is not 0, members below it will hold no role"
            );
        }

        Ok(config)
    }

    pub fn points_channel_id(&self, guild_id: u64) -> Option<u64> {
        self.guilds
            .iter()
            .find(|guild| guild.id == guild_id)
            .and_then(|guild| guild.points_channel_id)
    }
}
