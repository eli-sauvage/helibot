use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    // Boxed: this variant is several hundred bytes, and it would otherwise set the size
    // of every Result in the crate.
    #[error("configuration error: {0}")]
    Config(#[source] Box<config::ConfigError>),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("database migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("discord error: {0}")]
    Discord(#[source] Box<serenity::Error>),

    #[error("guild {0} is not in the cache")]
    GuildNotCached(u64),

    #[error("no points channel for guild {0}: set points_channel_id in helibot.toml, or create a channel named after point_channel_name")]
    PointsChannelNotFound(u64),
}

impl From<config::ConfigError> for Error {
    fn from(err: config::ConfigError) -> Self {
        Error::Config(Box::new(err))
    }
}

impl From<serenity::Error> for Error {
    fn from(err: serenity::Error) -> Self {
        Error::Discord(Box::new(err))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
