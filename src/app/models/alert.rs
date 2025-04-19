use crate::model_redis_impl;
use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs, Value};
use serde_bare;
use serde_derive::{Deserialize, Serialize};

pub static ALERTS_KEY: &str = "alerts";

#[derive(Serialize, Deserialize, Debug)]
pub struct Alert {
    pub style: String,
    pub content: String,
}

impl Alert {
    pub fn new(style: String, content: String) -> Self {
        Self { style, content }
    }
    pub fn info(content: String) -> Self {
        Self::new("info".to_string(), content)
    }
    pub fn success(content: String) -> Self {
        Self::new("success".to_string(), content)
    }
    pub fn warning(content: String) -> Self {
        Self::new("warning".to_string(), content)
    }
    pub fn error(content: String) -> Self {
        Self::new("error".to_string(), content)
    }
}

model_redis_impl!(Alert);
