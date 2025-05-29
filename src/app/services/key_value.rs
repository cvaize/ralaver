use crate::redis_connection::RedisPool;
use actix_web::web::Data;
use r2d2::PooledConnection;
use redis::{Client, Commands, Expiry, FromRedisValue, RedisError, ToRedisArgs};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone)]
pub struct KeyValueService {
    pool: Data<RedisPool>,
}

// TODO: Мысль: а что если redis будет внутри приложения и обращения к нему будут через память, а не через сетевой интерфейс.
impl KeyValueService {
    pub fn new(pool: Data<RedisPool>) -> Self {
        Self { pool }
    }

    pub fn get_connection(&self) -> Result<KeyValueConnection, KeyValueServiceError> {
        let conn: PooledConnection<Client> = self.pool.get_ref().get().map_err(|e| {
            log::error!("KeyValueService::ConnectFail - {e}");
            KeyValueServiceError::ConnectFail
        })?;

        Ok(KeyValueConnection::new(conn))
    }

    pub fn get<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        self.get_connection()?.get(key)
    }

    pub fn get_ex<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
        seconds: u64,
    ) -> Result<Option<V>, KeyValueServiceError> {
        self.get_connection()?.get_ex(key, seconds)
    }

    pub fn get_del<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        self.get_connection()?.get_del(key)
    }

    pub fn set<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        value: V,
    ) -> Result<(), KeyValueServiceError> {
        self.get_connection()?.set(key, value)
    }

    pub fn set_ex<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        value: V,
        seconds: u64,
    ) -> Result<(), KeyValueServiceError> {
        self.get_connection()?.set_ex(key, value, seconds)
    }

    pub fn expire<K: ToRedisArgs>(&self, key: K, seconds: i64) -> Result<(), KeyValueServiceError> {
        self.get_connection()?.expire(key, seconds)
    }

    pub fn del<K: ToRedisArgs>(&self, key: K) -> Result<(), KeyValueServiceError> {
        self.get_connection()?.del(key)
    }

    pub fn incr<K: ToRedisArgs, D: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
        delta: D,
    ) -> Result<V, KeyValueServiceError> {
        self.get_connection()?.incr(key, delta)
    }

    pub fn ttl<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
    ) -> Result<V, KeyValueServiceError> {
        self.get_connection()?.ttl(key)
    }
}

pub struct KeyValueConnection {
    conn: PooledConnection<Client>,
}

impl KeyValueConnection {
    pub fn new(conn: PooledConnection<Client>) -> Self {
        Self { conn }
    }

    pub fn get<K: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        let value = self.conn.get(key).map_err(|e| {
            log::error!("KeyValueService::get - {e}");
            KeyValueServiceError::Fail
        })?;
        Ok(value)
    }

    pub fn get_ex<K: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
        seconds: u64,
    ) -> Result<Option<V>, KeyValueServiceError> {
        let value = self.conn.get_ex(key, Expiry::EX(seconds)).map_err(|e| {
            log::error!("KeyValueService::get_ex - {e}");
            KeyValueServiceError::Fail
        })?;
        Ok(value)
    }

    pub fn get_del<K: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        let value = self.conn.get_del(&key).map_err(|e| {
            log::error!("KeyValueService::get_del - {e}");
            KeyValueServiceError::Fail
        })?;
        Ok(value)
    }

    pub fn set<K: ToRedisArgs, V: ToRedisArgs>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<(), KeyValueServiceError> {
        let result: Result<String, RedisError> = self.conn.set(&key, value);
        if let Err(e) = result {
            log::error!("KeyValueService::set - {e}");
            if e.to_string() == "An error was signalled by the server - ResponseError: wrong number of arguments for 'set' command" {
                self.del(&key)?;
            } else {
                return Err(KeyValueServiceError::Fail);
            }
        }
        Ok(())
    }

    pub fn set_ex<K: ToRedisArgs, V: ToRedisArgs>(
        &mut self,
        key: K,
        value: V,
        seconds: u64,
    ) -> Result<(), KeyValueServiceError> {
        let result: Result<String, RedisError> = self.conn.set_ex(&key, value, seconds);
        if let Err(e) = result {
            log::error!("KeyValueService::set_ex - {e}");
            if e.to_string() == "An error was signalled by the server - ResponseError: wrong number of arguments for 'setex' command" {
                self.del(&key)?;
            } else {
                return Err(KeyValueServiceError::Fail);
            }
        }
        Ok(())
    }

    pub fn expire<K: ToRedisArgs>(
        &mut self,
        key: K,
        seconds: i64,
    ) -> Result<(), KeyValueServiceError> {
        self.conn.expire(key, seconds).map_err(|e| {
            log::error!("KeyValueService::expire - {e}");
            KeyValueServiceError::Fail
        })
    }

    pub fn del<K: ToRedisArgs>(&mut self, key: K) -> Result<(), KeyValueServiceError> {
        self.conn.del(key).map_err(|e| {
            log::error!("KeyValueService::del - {e}");
            KeyValueServiceError::Fail
        })
    }

    pub fn incr<K: ToRedisArgs, D: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
        delta: D,
    ) -> Result<V, KeyValueServiceError> {
        self.conn.incr(key, delta).map_err(|e| {
            log::error!("KeyValueService::incr - {e}");
            KeyValueServiceError::Fail
        })
    }

    pub fn ttl<K: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
    ) -> Result<V, KeyValueServiceError> {
        self.conn.ttl(key).map_err(|e| {
            log::error!("KeyValueService::ttl - {e}");
            KeyValueServiceError::Fail
        })
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum KeyValueServiceError {
    ConnectFail,
    Fail,
}

#[cfg(test)]
mod tests {
    use crate::preparation;

    #[test]
    fn test() {
        let (_, all_services) = preparation();
        let key_value_service = all_services.key_value_service.get_ref();
        let key = "app::services::key_value::tests::get_and_set_and_del";
        let v: u64 = 123;
        key_value_service.del(key).unwrap();
        let value: i64 = key_value_service.ttl(key).unwrap();
        assert_eq!(value, -2);

        let value: Option<u64> = key_value_service.get(key).unwrap();
        assert!(value.is_none());

        key_value_service.set(key, v).unwrap();
        let value: i64 = key_value_service.ttl(key).unwrap();
        assert_eq!(value, -1);

        let value: Option<u64> = key_value_service.get_ex(key, 600).unwrap();
        assert!(value.is_some());

        let value: i64 = key_value_service.ttl(key).unwrap();
        assert!(value > 0);

        key_value_service.set_ex(key, v, 600).unwrap();
        let value: i64 = key_value_service.ttl(key).unwrap();
        assert!(value > 0);

        let value: u64 = key_value_service.incr(key, 1).unwrap();
        assert_eq!(value - 1, v);
        let value: Option<u64> = key_value_service.get(key).unwrap();
        assert_eq!(value.unwrap() - 1, v);

        key_value_service.expire(key, 600).unwrap();
        let value: Option<u64> = key_value_service.get(key).unwrap();
        assert!(value.is_some());
        let value: Option<u64> = key_value_service.get_del(key).unwrap();
        assert!(value.is_some());
        assert_eq!(value.unwrap(), v + 1);
    }
}
