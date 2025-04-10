use crate::Config;
use r2d2::Pool;
use redis::{Client, Commands};

pub type RedisPool = Pool<Client>;

pub fn get_connection_pool(config: &Config) -> RedisPool {
    log::info!("Connecting to the redis database.");
    let database_url = config.db.redis.url.to_owned();
    let client = Client::open(database_url).expect("Failed to create redis Client.");

    Pool::builder()
        .build(client)
        .expect("Failed to create redis Pool.")
}
