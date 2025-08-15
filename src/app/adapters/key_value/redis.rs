use crate::helpers::{value_from_bytes, BytesValue};
use crate::{
    AppError, ExpirableKeyValueRepository, IncrementableKeyValueRepository, KeyValueRepository,
    RedisRepository,
};

pub struct RedisRepositoryKeyValueAdapter {
    repository: RedisRepository,
}

impl RedisRepositoryKeyValueAdapter {
    pub fn new(repository: RedisRepository) -> Self {
        Self { repository }
    }
}

impl KeyValueRepository for RedisRepositoryKeyValueAdapter {
    fn get<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError> {
        value_from_bytes(self.repository.get(key)?)
    }

    fn get_del<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError> {
        value_from_bytes(self.repository.get_del(key)?)
    }

    fn set<V: BytesValue>(&self, key: &str, value: V) -> Result<(), AppError> {
        self.repository.set(key, value.value_to_bytes()?)?;
        Ok(())
    }

    fn del(&self, key: &str) -> Result<(), AppError> {
        self.repository.del(key)?;
        Ok(())
    }
}

impl IncrementableKeyValueRepository for RedisRepositoryKeyValueAdapter {
    fn incr(&self, key: &str, delta: i64) -> Result<i64, AppError> {
        self.repository.incr(key, delta)
    }
}

impl ExpirableKeyValueRepository for RedisRepositoryKeyValueAdapter {
    fn get_ex<V: BytesValue>(&self, key: &str, seconds: u64) -> Result<Option<V>, AppError> {
        value_from_bytes(self.repository.get_ex(key, seconds)?)
    }

    fn set_ex<V: BytesValue>(&self, key: &str, value: V, seconds: u64) -> Result<(), AppError> {
        self.repository
            .set_ex(key, value.value_to_bytes()?, seconds)?;
        Ok(())
    }

    fn expire(&self, key: &str, seconds: u64) -> Result<(), AppError> {
        self.repository.expire(key, seconds)?;
        Ok(())
    }

    fn ttl(&self, key: &str) -> Result<u64, AppError> {
        self.repository.ttl(key)
    }
}
