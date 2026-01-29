use serde::Deserialize;
use config::{Config as Cfg, ConfigError, File};
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: Server,
    pub database: Database,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub workers: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
    pub pool_size: u32,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let run_env = env::var("RUN_ENV").unwrap_or_else(|_| "dev".into());

        let builder = Cfg::builder()
            .add_source(File::with_name("config/default.yaml"))
            .add_source(File::with_name(&format!("config/{}", run_env)).required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"));
        builder.build()?.try_deserialize()
    }
}

