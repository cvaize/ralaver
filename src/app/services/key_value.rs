use crate::helpers::BytesValue;
use crate::{
    AppError, ExpirableKeyValueRepository, IncrementableKeyValueRepository, KeyValueRepository,
    KeyValueRepositoryType,
};
use actix_web::web::Data;

pub struct KeyValueService {
    repository: Data<KeyValueRepositoryType>,
}

impl KeyValueService {
    pub fn new(repository: Data<KeyValueRepositoryType>) -> Self {
        Self { repository }
    }

    pub fn get<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError> {
        self.repository.get_ref().get(key)
    }
    // Get and delete
    pub fn get_del<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError> {
        self.repository.get_ref().get_del(key)
    }
    pub fn set<V: BytesValue>(&self, key: &str, value: V) -> Result<(), AppError> {
        self.repository.get_ref().set(key, value)
    }
    pub fn del(&self, key: &str) -> Result<(), AppError> {
        self.repository.get_ref().del(key)
    }

    /// Increment the numeric value of a key by the given amount.
    /// If the key does not exist, it is set to 0 before performing the operation.
    /// Returns the current value in the response.
    pub fn incr(&self, key: &str, delta: i64) -> Result<i64, AppError> {
        self.repository.get_ref().incr(key, delta)
    }

    // Get and set expires
    pub fn get_ex<V: BytesValue>(&self, key: &str, seconds: u64) -> Result<Option<V>, AppError> {
        self.repository.get_ref().get_ex(key, seconds)
    }
    // Set data and expires
    pub fn set_ex<V: BytesValue>(&self, key: &str, value: V, seconds: u64) -> Result<(), AppError> {
        self.repository.get_ref().set_ex(key, value, seconds)
    }
    pub fn expire(&self, key: &str, seconds: u64) -> Result<(), AppError> {
        self.repository.get_ref().expire(key, seconds)
    }
    /// Get the time to live for a key in seconds.
    pub fn ttl(&self, key: &str) -> Result<u64, AppError> {
        self.repository.get_ref().ttl(key)
    }
}
