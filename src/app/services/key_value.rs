use crate::redis_connection::RedisPool;
use crate::LogService;
use actix_web::web::Data;
use redis::{Commands, FromRedisValue, ToRedisArgs};
use strum_macros::{Display, EnumString};
// https://docs.rs/redis/latest/redis/#type-conversions

pub struct KeyValueService {
    pool: RedisPool,
    log_service: Data<LogService>,
}

impl KeyValueService {
    pub fn new(pool: RedisPool, log_service: Data<LogService>) -> Self {
        Self { pool, log_service }
    }

    pub fn get<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        let mut conn = self.pool.get().map_err(|e| {
            self.log_service
                .get_ref()
                .error(format!("KeyValueService::get - {:}", &e).as_str());
            KeyValueServiceError::ConnectFail
        })?;
        let value = conn.get(key).map_err(|e| {
            self.log_service
                .get_ref()
                .error(format!("KeyValueService::get - {:}", &e).as_str());
            KeyValueServiceError::GetFail
        })?;
        Ok(value)
    }

    pub fn set<K: ToRedisArgs, V: ToRedisArgs, RV: FromRedisValue>(
        &self,
        key: K,
        value: V,
    ) -> Result<RV, KeyValueServiceError> {
        let mut conn = self.pool.get().map_err(|e| {
            self.log_service
                .get_ref()
                .error(format!("KeyValueService::set - {:}", &e).as_str());
            KeyValueServiceError::ConnectFail
        })?;
        Ok(conn.set(key, value).map_err(|e| {
            self.log_service
                .get_ref()
                .error(format!("KeyValueService::set - {:}", &e).as_str());
            KeyValueServiceError::SetFail
        })?)
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum KeyValueServiceError {
    ConnectFail,
    SetFail,
    GetFail,
}
