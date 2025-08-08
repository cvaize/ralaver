mod redis_key_value;

pub use self::redis_key_value::*;

use crate::helpers::{BytesKey, BytesValue};
use crate::AppError;

pub trait KeyValueAdapter {
    fn get<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError>;
    // Get and delete
    fn get_del<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError>;
    fn set<V: BytesValue>(&self, key: &str, value: V) -> Result<(), AppError>;
    fn del(&self, key: &str) -> Result<(), AppError>;
}

pub trait ConnectableKeyValueAdapter {
    fn get_connection<C: KeyValueAdapter>(&self) -> Result<C, AppError>;
}

pub trait IncrementableKeyValueAdapter {
    // Increment or decrement if minus exists
    fn incr(&self, key: &str, delta: i64) -> Result<Vec<u8>, AppError>;
}

pub trait ExpirableKeyValueAdapter {
    // Get and set expires
    fn get_ex<V: BytesValue>(&self, key: &str, seconds: u64) -> Result<Option<V>, AppError>;
    // Set data and expires
    fn set_ex<V: BytesValue>(&self, key: &str, value: V, seconds: u64) -> Result<(), AppError>;
    fn expire(&self, key: &str, seconds: u64) -> Result<(), AppError>;
    fn ttl(&self, key: &str) -> Result<u64, AppError>;
}
