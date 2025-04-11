use crate::{Config, LogService};
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::Key;
use r2d2::Pool;
use redis::Client;
use strum_macros::{Display, EnumString};

pub type RedisPool = Pool<Client>;

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum RedisConnectionError {
    CreateClientFail,
    CreatePoolFail,
    GetSessionStoreFail,
}

pub fn get_connection_pool(
    config: &Config,
    log_service: &LogService,
) -> Result<RedisPool, RedisConnectionError> {
    log_service.info("Connecting to Redis database.");
    let database_url = config.db.redis.url.to_owned();

    let client = Client::open(database_url).map_err(|e| {
        log_service.error(format!("RedisConnectionError::CreateClientFail - {:}", &e).as_str());
        RedisConnectionError::CreateClientFail
    })?;

    Pool::builder().build(client).map_err(|e| {
        log_service.error(format!("RedisConnectionError::CreatePoolFail - {:}", &e).as_str());
        RedisConnectionError::CreatePoolFail
    })
}

pub fn get_session_secret(config: &Config) -> Key {
    Key::from(config.db.redis.secret.to_owned().as_bytes())
}

pub async fn get_session_store(
    config: &Config,
    log_service: &LogService,
) -> Result<RedisSessionStore, RedisConnectionError> {
    RedisSessionStore::new(config.db.redis.url.to_owned())
        .await
        .map_err(|e| {
            log_service
                .error(format!("RedisConnectionError::GetSessionStoreFail - {:}", &e).as_str());
            RedisConnectionError::GetSessionStoreFail
        })
}
