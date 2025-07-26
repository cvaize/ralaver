use crate::redis_connection::RedisPool;
use actix_web::web::Data;
use r2d2::PooledConnection;
use redis::{Client, Commands, Expiry, FromRedisValue, RedisError, ToRedisArgs};
use crate::AppError;

#[derive(Debug, Clone)]
pub struct RedisRepository {
    pool: Data<RedisPool>,
}

impl RedisRepository {
    pub fn new(pool: Data<RedisPool>) -> Self {
        Self { pool }
    }

    pub fn get_connection(&self) -> Result<RedisRepositoryConnection, AppError> {
        let conn: PooledConnection<Client> = self.pool.get_ref().get().map_err(|e| {
            log::error!("RedisRepository::ConnectFail - {e}");
            AppError(Some(e.to_string()))
        })?;

        Ok(RedisRepositoryConnection::new(conn))
    }

    pub fn get<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
    ) -> Result<Option<V>, AppError> {
        self.get_connection()?.get(key)
    }

    pub fn get_ex<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
        seconds: u64,
    ) -> Result<Option<V>, AppError> {
        self.get_connection()?.get_ex(key, seconds)
    }

    pub fn set<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        value: V,
    ) -> Result<(), AppError> {
        self.get_connection()?.set(key, value)
    }

    pub fn set_ex<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        value: V,
        seconds: u64,
    ) -> Result<(), AppError> {
        self.get_connection()?.set_ex(key, value, seconds)
    }

    pub fn expire<K: ToRedisArgs>(&self, key: K, seconds: i64) -> Result<(), AppError> {
        self.get_connection()?.expire(key, seconds)
    }

    pub fn del<K: ToRedisArgs>(&self, key: K) -> Result<(), AppError> {
        self.get_connection()?.del(key)
    }

    pub fn incr<K: ToRedisArgs, D: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
        delta: D,
    ) -> Result<V, AppError> {
        self.get_connection()?.incr(key, delta)
    }

    pub fn ttl<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
    ) -> Result<V, AppError> {
        self.get_connection()?.ttl(key)
    }
}

pub struct RedisRepositoryConnection {
    conn: PooledConnection<Client>,
}

impl RedisRepositoryConnection {
    pub fn new(conn: PooledConnection<Client>) -> Self {
        Self { conn }
    }

    pub fn get<K: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
    ) -> Result<Option<V>, AppError> {
        let value = self.conn.get(key).map_err(|e| {
            log::error!("RedisRepository::get - {e}");
            AppError(Some(e.to_string()))
        })?;
        Ok(value)
    }

    pub fn get_ex<K: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
        seconds: u64,
    ) -> Result<Option<V>, AppError> {
        let value = self.conn.get_ex(key, Expiry::EX(seconds)).map_err(|e| {
            log::error!("RedisRepository::get_ex - {e}");
            AppError(Some(e.to_string()))
        })?;
        Ok(value)
    }

    pub fn get_del<K: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
    ) -> Result<Option<V>, AppError> {
        let value = self.conn.get_del(&key).map_err(|e| {
            log::error!("RedisRepository::get_del - {e}");
            AppError(Some(e.to_string()))
        })?;
        Ok(value)
    }

    pub fn set<K: ToRedisArgs, V: ToRedisArgs>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<(), AppError> {
        let result: Result<String, RedisError> = self.conn.set(&key, value);
        if let Err(e) = result {
            log::error!("RedisRepository::set - {e}");
            if e.to_string() == "An error was signalled by the server - ResponseError: wrong number of arguments for 'set' command" {
                self.del(&key)?;
            } else {
                return Err(AppError(Some(e.to_string())));
            }
        }
        Ok(())
    }

    pub fn set_ex<K: ToRedisArgs, V: ToRedisArgs>(
        &mut self,
        key: K,
        value: V,
        seconds: u64,
    ) -> Result<(), AppError> {
        let result: Result<String, RedisError> = self.conn.set_ex(&key, value, seconds);
        if let Err(e) = result {
            log::error!("RedisRepository::set_ex - {e}");
            if e.to_string() == "An error was signalled by the server - ResponseError: wrong number of arguments for 'setex' command" {
                self.del(&key)?;
            } else {
                return Err(AppError(Some(e.to_string())));
            }
        }
        Ok(())
    }

    pub fn expire<K: ToRedisArgs>(
        &mut self,
        key: K,
        seconds: i64,
    ) -> Result<(), AppError> {
        self.conn.expire(key, seconds).map_err(|e| {
            log::error!("RedisRepository::expire - {e}");
            AppError(Some(e.to_string()))
        })
    }

    pub fn del<K: ToRedisArgs>(&mut self, key: K) -> Result<(), AppError> {
        self.conn.del(key).map_err(|e| {
            log::error!("RedisRepository::del - {e}");
            AppError(Some(e.to_string()))
        })
    }

    pub fn incr<K: ToRedisArgs, D: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
        delta: D,
    ) -> Result<V, AppError> {
        self.conn.incr(key, delta).map_err(|e| {
            log::error!("RedisRepository::incr - {e}");
            AppError(Some(e.to_string()))
        })
    }

    pub fn ttl<K: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
    ) -> Result<V, AppError> {
        self.conn.ttl(key).map_err(|e| {
            log::error!("RedisRepository::ttl - {e}");
            AppError(Some(e.to_string()))
        })
    }
}
