use crate::helpers::BytesValue;
use crate::{AppError, RedisRepository};
use actix_web::web::Data;
use redis::{FromRedisValue, ToRedisArgs};

pub struct KeyValueService {
    repository: Data<RedisRepository>,
}

impl KeyValueService {
    pub fn new(repository: Data<RedisRepository>) -> Self {
        Self { repository }
    }

    pub fn get<K: ToRedisArgs, V: FromRedisValue>(&self, key: K) -> Result<Option<V>, AppError> {
        self.repository.get_ref().get(key)
    }
    // Get and delete
    pub fn get_del<K: ToRedisArgs, V: FromRedisValue>(&self, key: K) -> Result<Option<V>, AppError> {
        self.repository.get_ref().get_del(key)
    }
    pub fn set<K: ToRedisArgs, V: ToRedisArgs>(&self, key: K, value: V) -> Result<(), AppError> {
        self.repository.get_ref().set(key, value)
    }
    pub fn del<K: ToRedisArgs>(&self, key: K) -> Result<(), AppError> {
        self.repository.get_ref().del(key)
    }

    /// Increment the numeric value of a key by the given amount.
    /// If the key does not exist, it is set to 0 before performing the operation.
    /// Returns the current value in the response.
    pub fn incr<K: ToRedisArgs, D: ToRedisArgs, V: FromRedisValue>(&self, key: K, delta: D) -> Result<V, AppError> {
        self.repository.get_ref().incr(key, delta)
    }

    // Get and set expires
    pub fn get_ex<K: ToRedisArgs, V: FromRedisValue>(&self, key: K, seconds: u64) -> Result<Option<V>, AppError> {
        self.repository.get_ref().get_ex(key, seconds)
    }
    // Set data and expires
    pub fn set_ex<K: ToRedisArgs, V: ToRedisArgs>(&self, key: K, value: V, seconds: u64) -> Result<(), AppError> {
        self.repository.get_ref().set_ex(key, value, seconds)
    }
    pub fn expire<K: ToRedisArgs>(&self, key: K, seconds: u64) -> Result<(), AppError> {
        self.repository.get_ref().expire(key, seconds)
    }
    /// Get the time to live for a key in seconds.
    pub fn ttl<K: ToRedisArgs>(&self, key: K) -> Result<u64, AppError> {
        self.repository.get_ref().ttl(key)
    }
}
