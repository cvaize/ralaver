use crate::redis_connection::RedisPool;
use redis::{Commands, FromRedisValue, ToRedisArgs};

// https://docs.rs/redis/latest/redis/#type-conversions

pub struct KeyValueService {
    pool: RedisPool,
}

impl KeyValueService {
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    pub fn get<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|_| KeyValueServiceError::ConnectFail)?;
        let value = conn.get(key).map_err(|_| KeyValueServiceError::GetFail)?;
        Ok(value)
    }

    pub fn set<K: ToRedisArgs, V: ToRedisArgs, RV: FromRedisValue>(
        &self,
        key: K,
        value: V,
    ) -> Result<RV, KeyValueServiceError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|_| KeyValueServiceError::ConnectFail)?;
        Ok(conn
            .set(key, value)
            .map_err(|_| KeyValueServiceError::SetFail)?)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KeyValueServiceError {
    ConnectFail,
    SetFail,
    GetFail,
}
