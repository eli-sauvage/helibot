use thiserror::Error;

#[derive(Error, Debug)]
pub enum HelibotError {
    #[error("Sqlx error")]
    Sqlx(#[from] sqlx::error::Error),
    #[error("Migration error")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    EnvVarError(EnvVarError),
    #[error("points channel not found in guild {0}")]
    PointChannelNotFound(u64),
    #[error(transparent)]
    SerenityError(#[from] serenity::all::Error),
    #[error("config error")]
    ConfigError(#[from] config::ConfigError),
}

#[derive(Error, Debug)]
pub enum EnvVarError {
    //#[error("could not convert os string to string")]
    //OsString(std::ffi::OsString),
    //#[error("the variable was not found")]
    //VarNotFound,
    #[error("dotenv module failed to load env")]
    DotEnvModuleError(#[from] dotenvy::Error),
}

#[derive(Error, Debug)]

pub enum ConfError {}
