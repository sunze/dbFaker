use once_cell::sync::Lazy;
use mysql::{Pool, Opts, PooledConn};
use crate::types::conf::Config;

// 全局 MySQL 池
pub static DB_POOL: Lazy<Pool> = Lazy::new(|| {
    let config = Config::new().expect("Failed to load config");
    let opts = Opts::from_url(&config.database.url).expect("Invalid DB URL");
    println!("Server running at {}:{}", config.server.host, config.server.port);
    println!("DB url: {}", config.database.url);
    Pool::new(opts).expect("Failed to create MySQL pool")
});

// 对外暴露 API
pub fn get_pool() -> &'static Pool {
    &DB_POOL
}

pub fn get_conn() -> PooledConn {
    DB_POOL.get_conn().expect("Failed to get MySQL connection")
}