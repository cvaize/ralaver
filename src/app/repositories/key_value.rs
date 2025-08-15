use crate::helpers::BytesValue;
use crate::{AppError};

pub type KeyValueRepositoryType = crate::KVRepositoryKeyValueAdapter<'static>;
// pub type KeyValueRepositoryType = crate::RedisRepositoryKeyValueAdapter;

pub trait KeyValueRepository {
    fn get<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError>;
    // Get and delete
    fn get_del<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError>;
    fn set<V: BytesValue>(&self, key: &str, value: V) -> Result<(), AppError>;
    fn del(&self, key: &str) -> Result<(), AppError>;
}

pub trait IncrementableKeyValueRepository {
    /// Increment the numeric value of a key by the given amount.
    /// If the key does not exist, it is set to 0 before performing the operation.
    /// Returns the current value in the response.
    fn incr(&self, key: &str, delta: i64) -> Result<i64, AppError>;
}

pub trait ExpirableKeyValueRepository {
    // Get and set expires
    fn get_ex<V: BytesValue>(&self, key: &str, seconds: u64) -> Result<Option<V>, AppError>;
    // Set data and expires
    fn set_ex<V: BytesValue>(&self, key: &str, value: V, seconds: u64) -> Result<(), AppError>;
    fn expire(&self, key: &str, seconds: u64) -> Result<(), AppError>;
    /// Get the time to live for a key in seconds.
    fn ttl(&self, key: &str) -> Result<u64, AppError>;
}
