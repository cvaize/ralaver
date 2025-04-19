use crate::redis_connection::RedisPool;
use actix_web::web::Data;
use redis::{Commands, Expiry, FromRedisValue, RedisError, ToRedisArgs};
use strum_macros::{Display, EnumString};


#[derive(Debug, Clone)]
pub struct KeyValueService {
    pool: Data<RedisPool>,
}

impl KeyValueService {
    pub fn new(pool: Data<RedisPool>) -> Self {
        Self { pool }
    }

    pub fn get<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        let mut conn = self.pool.get_ref().get().map_err(|e| {
            log::error!("{}",format!("KeyValueService::ConnectFail - {:}", &e).as_str());
            KeyValueServiceError::ConnectFail
        })?;
        let value = conn.get(key).map_err(|e| {
            log::error!("{}",format!("KeyValueService::get - {:}", &e).as_str());
            KeyValueServiceError::GetFail
        })?;
        Ok(value)
    }

    pub fn get_ex<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
        seconds: u64,
    ) -> Result<Option<V>, KeyValueServiceError> {
        let mut conn = self.pool.get_ref().get().map_err(|e| {
            log::error!("{}",format!("KeyValueService::ConnectFail - {:}", &e).as_str());
            KeyValueServiceError::ConnectFail
        })?;
        let value = conn.get_ex(key, Expiry::EX(seconds)).map_err(|e| {
            log::error!("{}",format!("KeyValueService::get_ex - {:}", &e).as_str());
            KeyValueServiceError::GetExFail
        })?;
        Ok(value)
    }

    pub fn get_del<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        let mut conn = self.pool.get_ref().get().map_err(|e| {
            log::error!("{}",format!("KeyValueService::ConnectFail - {:}", &e).as_str());
            KeyValueServiceError::ConnectFail
        })?;
        let value = conn.get_del(&key).map_err(|e| {
            log::error!("{}",format!("KeyValueService::get_del - {:}", &e).as_str());
            KeyValueServiceError::GetDelFail
        })?;
        Ok(value)
    }

    pub fn set<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        value: V,
    ) -> Result<(), KeyValueServiceError> {
        let mut conn = self.pool.get_ref().get().map_err(|e| {
            log::error!("{}",format!("KeyValueService::ConnectFail - {:}", &e).as_str());
            KeyValueServiceError::ConnectFail
        })?;
        // conn.set(&key, value).map_err(|e| {
        //     log::error!("{}",format!("KeyValueService::set - {:}", &e).as_str());
        //     KeyValueServiceError::SetFail
        // })?;

        let result: Result<String, RedisError> = conn.set(&key, value);
        if let Err(e) = result {
            log::error!("{}",format!("KeyValueService::set - {:}", &e).as_str());
            if e.to_string() == "An error was signalled by the server - ResponseError: wrong number of arguments for 'set' command" {
                self.del(&key)?;
            } else {
                return Err(KeyValueServiceError::SetFail);
            }
        }
        Ok(())
    }

    pub fn set_ex<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        value: V,
        seconds: u64,
    ) -> Result<(), KeyValueServiceError> {
        let mut conn = self.pool.get_ref().get().map_err(|e| {
            log::error!("{}",format!("KeyValueService::ConnectFail - {:}", &e).as_str());
            KeyValueServiceError::ConnectFail
        })?;
        let result: Result<String, RedisError> = conn.set_ex(&key, value, seconds);
        if let Err(e) = result {
            log::error!("{}",format!("KeyValueService::set_ex - {:}", &e).as_str());
            if e.to_string() == "An error was signalled by the server - ResponseError: wrong number of arguments for 'setex' command" {
                self.del(&key)?;
            } else {
                return Err(KeyValueServiceError::SetExFail);
            }
        }
        // conn.set_ex(key, value, seconds).map_err(|e| {
        //     log::error!("{}",format!("KeyValueService::set_ex - {:}", &e).as_str());
        //     KeyValueServiceError::SetExFail
        // })?;
        Ok(())
    }

    pub fn expire<K: ToRedisArgs>(&self, key: K, seconds: i64) -> Result<(), KeyValueServiceError> {
        let mut conn = self.pool.get_ref().get().map_err(|e| {
            log::error!("{}",format!("KeyValueService::ConnectFail - {:}", &e).as_str());
            KeyValueServiceError::ConnectFail
        })?;
        conn.expire(key, seconds).map_err(|e| {
            log::error!("{}",format!("KeyValueService::expire - {:}", &e).as_str());
            KeyValueServiceError::ExpireFail
        })?;
        Ok(())
    }

    pub fn del<K: ToRedisArgs>(&self, key: K) -> Result<(), KeyValueServiceError> {
        let mut conn = self.pool.get_ref().get().map_err(|e| {
            log::error!("{}",format!("KeyValueService::ConnectFail - {:}", &e).as_str());
            KeyValueServiceError::ConnectFail
        })?;
        conn.del(key).map_err(|e| {
            log::error!("{}",format!("KeyValueService::del - {:}", &e).as_str());
            KeyValueServiceError::DelFail
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum KeyValueServiceError {
    ConnectFail,
    SetFail,
    SetExFail,
    GetFail,
    GetExFail,
    GetDelFail,
    DelFail,
    ExpireFail,
}
