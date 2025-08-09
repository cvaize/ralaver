use crate::helpers::{value_from_bytes, BytesValue};
use crate::{AppError, KeyValueRepository, KVRepository};
use actix_web::web::Data;
use image::EncodableLayout;

pub struct KVRepositoryKeyValueAdapter <'a>{
    rep: Data<KVRepository<'a>>,
}

impl <'a> KeyValueRepository for KVRepositoryKeyValueAdapter <'a>{
    fn get<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError> {
        value_from_bytes(self.rep.get_ref().get(key.as_bytes())?)
    }

    fn get_del<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError> {
        value_from_bytes(self.rep.get_ref().remove(key.as_bytes())?)
    }

    fn set<V: BytesValue>(&self, key: &str, value: V) -> Result<(), AppError> {
        self.rep.get_ref().set(key.as_bytes(), value.value_to_bytes()?.as_bytes())?;
        Ok(())
    }

    fn del(&self, key: &str) -> Result<(), AppError> {
        self.rep.get_ref().remove(key.as_bytes())?;
        Ok(())
    }
}