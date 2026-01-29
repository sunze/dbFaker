use std::env;
use serde::Deserialize;
use config::{Config as Cfg, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct AllRules {
    pub number: Option<NumberRule>,
    pub string: Option<StringRule>,
    pub date: Option<DateRule>,
    pub timestamp: Option<TimestampRule>,
}

#[derive(Debug, Deserialize)]
pub struct NumberRule {
    pub title: String,
    pub default: String,
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Deserialize)]
pub struct StringRule {
    pub title: String,
    pub default: String,
    pub pool_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct DateRule {
    pub title: String,
    pub default: String,
}

#[derive(Debug, Deserialize)]
pub struct TimestampRule {
    pub title: String,
    pub default: String,
}


impl AllRules {
    pub fn new() -> Result<Self, ConfigError> {
        let run_env = env::var("RUN_ENV").unwrap_or_else(|_| "dev".into());
    
        let builder = Cfg::builder()
            .add_source(File::with_name("config/rule"))
            .add_source(File::with_name(&format!("config/{}", run_env)).required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"));
    
        builder.build()?.try_deserialize()
    }
}