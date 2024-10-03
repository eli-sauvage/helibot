use serde_derive::Deserialize;
use serenity::prelude::TypeMapKey;

use config::{Config, Environment, File};

use crate::{
    errors::{EnvVarError, HelibotError},
    managers::roles::Seuil,
};

#[derive(Debug, Deserialize)]
pub struct Env {
    pub database_url: String,
    pub discord_token: String,
    pub point_channel_name: String,
    pub roles: Vec<Seuil>,
}

impl TypeMapKey for Env {
    type Value = Env;
}
impl Env {
    pub fn get_env() -> Result<Self, HelibotError> {
        dotenvy::dotenv()
            .map_err(|e| HelibotError::EnvVarError(EnvVarError::DotEnvModuleError(e)))?;
        let s = Config::builder()
            .add_source(File::with_name("helibot"))
            .add_source(Environment::with_prefix("HELIBOT"))
            // You may also programmatically change settings
            //.set_override("database.url", "postgres://")?
            .build()
            .map_err(HelibotError::ConfigError)?;

        // Now that we're done, let's access our configuration
        //println!("debug: {:?}", s.get_bool("debug"));
        //println!("database: {:?}", s.get::<String>("database.url"));

        // You can deserialize (and thus freeze) the entire configuration as
        let a: Env = s.try_deserialize().map_err(HelibotError::ConfigError)?;
        Ok(a)
    }

    pub fn get_discord_token(&self) -> &str {
        &self.discord_token
    }
}
