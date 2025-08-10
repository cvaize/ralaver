use crate::helpers::{now_timestamp, BytesValue};
use crate::{
    AppError, ExpirableKeyValueRepository, IncrementableKeyValueRepository, KVRepository,
    KeyValueRepository,
};
use actix_web::web::Data;

const TIMESTAMP_BYTES_MAX: [u8; 8] = [255, 255, 255, 255, 255, 255, 255, 255];
const TIMESTAMP_BYTES_LENGTH: usize = 8;

pub struct KVRepositoryKeyValueAdapter<'a> {
    repository: Data<KVRepository<'a>>,
}

impl<'a> KVRepositoryKeyValueAdapter<'a> {
    pub fn new(repository: Data<KVRepository<'a>>) -> Self {
        Self { repository }
    }

    /// Clearing expired values
    pub fn clean_expired_values(&mut self) -> Result<(), AppError> {
        let repository: &KVRepository = self.repository.get_ref();

        let mut error: Option<AppError> = None;
        for item in repository.iter() {
            if let Ok((key, value)) = item {
                let (timestamp, _) = value.split_at(TIMESTAMP_BYTES_LENGTH);
                let timestamp = u64::value_from_bytes(timestamp.to_vec());

                if let Ok(timestamp) = timestamp {
                    if timestamp <= now_timestamp() {
                        if let Err(e) = repository.remove(&key) {
                            error = Some(AppError(Some(e.to_string())));
                            log::error!("KVRepositoryKeyValueAdapter::clean_expired_values - {e}");
                        }
                    }
                } else if let Err(e) = timestamp {
                    error = Some(AppError(Some(e.to_string())));
                    log::error!("KVRepositoryKeyValueAdapter::clean_expired_values - {e}");
                }
            } else if let Err(e) = item {
                error = Some(AppError(Some(e.to_string())));
                log::error!("KVRepositoryKeyValueAdapter::clean_expired_values - {e}");
            }
        }

        if let Some(e) = error {
            return Err(e);
        }

        Ok(())
    }

    fn get_with_timestamp<V: BytesValue>(&self, key: &str) -> Result<Option<(V, u64)>, AppError> {
        let key: &[u8] = key.as_bytes();
        let repository: &KVRepository = self.repository.get_ref();

        let value: Option<Vec<u8>> = repository.get(key)?;
        if let Some(v) = value {
            let now: u64 = now_timestamp();
            let (timestamp, data) = v.split_at(TIMESTAMP_BYTES_LENGTH);
            let timestamp: u64 = u64::value_from_bytes(timestamp.to_vec())?;
            if timestamp <= now {
                repository.remove(key)?;
                return Ok(None);
            }
            return Ok(Some((V::value_from_bytes(data.to_vec())?, timestamp)));
        }

        Ok(None)
    }

    fn get_del_with_timestamp<V: BytesValue>(
        &self,
        key: &str,
    ) -> Result<Option<(V, u64)>, AppError> {
        let key: &[u8] = key.as_bytes();
        let repository: &KVRepository = self.repository.get_ref();

        let value: Option<Vec<u8>> = repository.remove(key)?;
        if let Some(v) = value {
            let now: u64 = now_timestamp();
            let (timestamp, data) = v.split_at(TIMESTAMP_BYTES_LENGTH);
            let timestamp: u64 = u64::value_from_bytes(timestamp.to_vec())?;
            if timestamp <= now {
                return Ok(None);
            }
            return Ok(Some((V::value_from_bytes(data.to_vec())?, timestamp)));
        }

        Ok(None)
    }
}

impl<'a> KeyValueRepository for KVRepositoryKeyValueAdapter<'a> {
    fn get<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError> {
        let value: Option<(V, u64)> = self.get_with_timestamp(key)?;

        if let Some(value) = value {
            return Ok(Some(value.0));
        }

        Ok(None)
    }

    fn get_del<V: BytesValue>(&self, key: &str) -> Result<Option<V>, AppError> {
        let value: Option<(V, u64)> = self.get_del_with_timestamp(key)?;

        if let Some(value) = value {
            return Ok(Some(value.0));
        }

        Ok(None)
    }

    fn set<V: BytesValue>(&self, key: &str, value: V) -> Result<(), AppError> {
        let mut data: Vec<u8> = value.value_to_bytes()?;
        let mut value: Vec<u8> = TIMESTAMP_BYTES_MAX.to_vec();
        value.append(&mut data);

        self.repository.get_ref().set(key.as_bytes(), &value)?;
        Ok(())
    }

    fn del(&self, key: &str) -> Result<(), AppError> {
        self.repository.get_ref().remove(key.as_bytes())?;
        Ok(())
    }
}

impl<'a> IncrementableKeyValueRepository for KVRepositoryKeyValueAdapter<'a> {
    fn incr(&self, key: &str, delta: i64) -> Result<i64, AppError> {
        let mut value: (i64, u64) = self.get_with_timestamp(key)?.unwrap_or((0, u64::MAX));
        value.0 += delta;
        self.set_ex(key, value.0, value.1)?;
        Ok(value.0)
    }
}

impl<'a> ExpirableKeyValueRepository for KVRepositoryKeyValueAdapter<'a> {
    fn get_ex<V: BytesValue>(&self, key: &str, seconds: u64) -> Result<Option<V>, AppError> {
        let key: &[u8] = key.as_bytes();
        let repository: &KVRepository = self.repository.get_ref();

        let value: Option<Vec<u8>> = repository.get(key)?;
        if let Some(v) = value {
            let now: u64 = now_timestamp();
            let (timestamp, data) = v.split_at(TIMESTAMP_BYTES_LENGTH);
            let timestamp: u64 = u64::value_from_bytes(timestamp.to_vec())?;
            if timestamp <= now {
                repository.remove(key)?;
                return Ok(None);
            }
            let mut append_data: Vec<u8> = data.to_vec();
            let mut value: Vec<u8> = (now + seconds).value_to_bytes()?;
            value.append(&mut append_data);

            return Ok(Some(V::value_from_bytes(data.to_vec())?));
        }

        Ok(None)
    }

    fn set_ex<V: BytesValue>(&self, key: &str, value: V, seconds: u64) -> Result<(), AppError> {
        let mut data: Vec<u8> = value.value_to_bytes()?;
        let mut value: Vec<u8> = (now_timestamp() + seconds).value_to_bytes()?;
        value.append(&mut data);

        self.repository.get_ref().set(key.as_bytes(), &value)?;
        Ok(())
    }

    fn expire(&self, key: &str, seconds: u64) -> Result<(), AppError> {
        let key: &[u8] = key.as_bytes();
        let repository: &KVRepository = self.repository.get_ref();

        let value: Option<Vec<u8>> = repository.get(key)?;
        if let Some(v) = value {
            let (_, data) = v.split_at(TIMESTAMP_BYTES_LENGTH);
            let mut data: Vec<u8> = data.to_vec();

            let mut value: Vec<u8> = (now_timestamp() + seconds).value_to_bytes()?;
            value.append(&mut data);

            self.repository.get_ref().set(key, &value)?;
        }
        Ok(())
    }

    fn ttl(&self, key: &str) -> Result<u64, AppError> {
        let key: &[u8] = key.as_bytes();
        let repository: &KVRepository = self.repository.get_ref();

        let value: Option<Vec<u8>> = repository.get(key)?;
        if let Some(v) = value {
            let (timestamp, _) = v.split_at(TIMESTAMP_BYTES_LENGTH);
            let timestamp: u64 = u64::value_from_bytes(timestamp.to_vec())?;
            let now: u64 = now_timestamp();

            if timestamp <= now {
                repository.remove(key)?;
                return Ok(u64::MAX);
            }

            return Ok(timestamp - now);
        }

        Ok(u64::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::make_config;
    use test::Bencher;

    #[test]
    fn test_incrementable_key_value_repo_incr() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::adapters::key_value::kv::tests::test_incrementable_key_value_repo_incr
        let config = make_config();
        let kv_repository = Data::new(KVRepository::new(&config.db.kv.storage).unwrap());
        let kv_adapter = KVRepositoryKeyValueAdapter::new(kv_repository);

        let key = "test_incrementable_key_value_repo_incr";
        let mut value = kv_adapter.incr(key, 5).unwrap();
        assert_eq!(value, 5);
        value = kv_adapter.incr(key, 5).unwrap();
        assert_eq!(value, 10);
        value = kv_adapter.incr(key, -2).unwrap();
        assert_eq!(value, 8);
        value = kv_adapter.incr(key, -20).unwrap();
        assert_eq!(value, -12);
        value = kv_adapter.incr(key, 12).unwrap();
        assert_eq!(value, 0);
        kv_adapter.del(key).unwrap();
    }

    #[bench]
    fn bench_incrementable_key_value_repo_incr(b: &mut Bencher) {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo bench -- --nocapture --exact app::adapters::key_value::kv::tests::bench_incrementable_key_value_repo_incr
        // 1,249.39 ns/iter (+/- 29.61)
        let config = make_config();
        let kv_repository = Data::new(KVRepository::new(&config.db.kv.storage).unwrap());
        let kv_adapter = KVRepositoryKeyValueAdapter::new(kv_repository);

        let key = "bench_incrementable_key_value_repo_incr";
        b.iter(|| {
            let _ = kv_adapter.incr(key, 5).unwrap();
        });
        kv_adapter.del(key).unwrap();
    }

    #[test]
    fn test_clean_expired_values() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::adapters::key_value::kv::tests::test_clean_expired_values
        let config = make_config();
        let kv_repository = Data::new(KVRepository::new(&config.db.kv.storage).unwrap());
        let mut kv_adapter = KVRepositoryKeyValueAdapter::new(kv_repository);

        kv_adapter.clean_expired_values().unwrap();
    }

    #[test]
    fn test_ttl() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::adapters::key_value::kv::tests::test_ttl
    }
}
