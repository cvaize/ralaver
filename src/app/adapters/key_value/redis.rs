use crate::helpers::{value_from_bytes, BytesValue};
use crate::{
    AppError, ExpirableKeyValueRepository, IncrementableKeyValueRepository, KeyValueRepository,
    RedisRepository,
};
use actix_web::web::Data;

pub struct RedisRepositoryKeyValueAdapter {
    rep: Data<RedisRepository>,
}

impl RedisRepositoryKeyValueAdapter {
    pub fn new(rep: Data<RedisRepository>) -> Self {
        Self { rep }
    }
}

impl KeyValueRepository for RedisRepositoryKeyValueAdapter {
    fn get<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError> {
        value_from_bytes(self.rep.get_ref().get(key)?)
    }

    fn get_del<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError> {
        value_from_bytes(self.rep.get_ref().get_del(key)?)
    }

    fn set<V: BytesValue>(&self, key: &str, value: V) -> Result<(), AppError> {
        self.rep.get_ref().set(key, value.value_to_bytes()?)?;
        Ok(())
    }

    fn del(&self, key: &str) -> Result<(), AppError> {
        self.rep.get_ref().del(key)?;
        Ok(())
    }
}

impl IncrementableKeyValueRepository for RedisRepositoryKeyValueAdapter {
    fn incr(&self, key: &str, delta: i64) -> Result<i64, AppError> {
        self.rep.get_ref().incr(key, delta)
    }
}

impl ExpirableKeyValueRepository for RedisRepositoryKeyValueAdapter {
    fn get_ex<V: BytesValue>(&self, key: &str, seconds: u64) -> Result<Option<V>, AppError> {
        value_from_bytes(self.rep.get_ref().get_ex(key, seconds)?)
    }

    fn set_ex<V: BytesValue>(&self, key: &str, value: V, seconds: u64) -> Result<(), AppError> {
        self.rep
            .get_ref()
            .set_ex(key, value.value_to_bytes()?, seconds)?;
        Ok(())
    }

    fn expire(&self, key: &str, seconds: u64) -> Result<(), AppError> {
        self.rep.get_ref().expire(key, seconds)?;
        Ok(())
    }

    fn ttl(&self, key: &str) -> Result<u64, AppError> {
        self.rep.get_ref().ttl(key)
    }
}
