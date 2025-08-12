use crate::helpers::{now_timestamp, BytesValue};
use crate::{
    AppError, ExpirableKeyValueRepository, IncrementableKeyValueRepository, KVBucketRepository,
    KVRepository, KeyValueRepository,
};
use actix_web::web::Data;

const TIMESTAMP_BYTES_MAX: [u8; 8] = [255, 255, 255, 255, 255, 255, 255, 255];
const TIMESTAMP_BYTES_LENGTH: usize = 8;

pub struct KVRepositoryKeyValueAdapter<'a> {
    pub repository: Data<KVBucketRepository<'a>>,
}

impl<'a> KVRepositoryKeyValueAdapter<'a> {
    pub fn new(repository: Data<KVBucketRepository<'a>>) -> Self {
        Self { repository }
    }

    /// Clearing expired values
    pub fn clean_expired_values(&mut self) -> Result<(), AppError> {
        let repository: &KVBucketRepository = self.repository.get_ref();

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

    fn sum(&self, a: u64, b: u64) -> u64 {
        if a == u64::MAX || b == u64::MAX {
            return u64::MAX;
        }

        if b > (u64::MAX - a) {
            return u64::MAX;
        }
        a + b
    }

    fn get_with_timestamp<V: BytesValue>(&self, key: &str) -> Result<Option<(V, u64)>, AppError> {
        let key: &[u8] = key.as_bytes();
        let repository: &KVBucketRepository = self.repository.get_ref();

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
        let repository: &KVBucketRepository = self.repository.get_ref();

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
        let mut value_with_timestamp: (i64, u64) = self.get_with_timestamp(key)?.unwrap_or((0, u64::MAX));
        value_with_timestamp.0 += delta;

        let mut data: Vec<u8> = value_with_timestamp.0.value_to_bytes()?;
        let mut value: Vec<u8> = value_with_timestamp.1.value_to_bytes()?;
        value.append(&mut data);

        self.repository.get_ref().set(key.as_bytes(), &value)?;

        Ok(value_with_timestamp.0)
    }
}

impl<'a> ExpirableKeyValueRepository for KVRepositoryKeyValueAdapter<'a> {
    fn get_ex<V: BytesValue>(&self, key: &str, seconds: u64) -> Result<Option<V>, AppError> {
        let key: &[u8] = key.as_bytes();
        let repository: &KVBucketRepository = self.repository.get_ref();

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
            let mut value: Vec<u8> = self.sum(now, seconds).value_to_bytes()?;
            value.append(&mut append_data);
            self.repository.get_ref().set(key, &value)?;

            return Ok(Some(V::value_from_bytes(data.to_vec())?));
        }

        Ok(None)
    }

    fn set_ex<V: BytesValue>(&self, key: &str, value: V, seconds: u64) -> Result<(), AppError> {
        let mut data: Vec<u8> = value.value_to_bytes()?;
        let mut value: Vec<u8> = self.sum(now_timestamp(), seconds).value_to_bytes()?;
        value.append(&mut data);

        self.repository.get_ref().set(key.as_bytes(), &value)?;
        Ok(())
    }

    fn expire(&self, key: &str, seconds: u64) -> Result<(), AppError> {
        let key: &[u8] = key.as_bytes();
        let repository: &KVBucketRepository = self.repository.get_ref();

        let value: Option<Vec<u8>> = repository.get(key)?;
        if let Some(v) = value {
            let (_, data) = v.split_at(TIMESTAMP_BYTES_LENGTH);
            let mut data: Vec<u8> = data.to_vec();

            let mut value: Vec<u8> = self.sum(now_timestamp(), seconds).value_to_bytes()?;
            value.append(&mut data);

            self.repository.get_ref().set(key, &value)?;
        }
        Ok(())
    }

    fn ttl(&self, key: &str) -> Result<u64, AppError> {
        let key: &[u8] = key.as_bytes();
        let repository: &KVBucketRepository = self.repository.get_ref();

        let value: Option<Vec<u8>> = repository.get(key)?;
        if let Some(v) = value {
            let (timestamp, _) = v.split_at(TIMESTAMP_BYTES_LENGTH);
            let timestamp: u64 = u64::value_from_bytes(timestamp.to_vec())?;

            if timestamp == u64::MAX {
                return Ok(u64::MAX);
            }

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

    struct TestAdapterData<'a> {
        kv_repository: KVRepository<'a>,
        kv_adapter: KVRepositoryKeyValueAdapter<'a>,
    }

    fn make_adapter<'a>(name: &str) -> TestAdapterData<'a> {
        let config = make_config();
        let kv_repository = KVRepository::new(&config.db.kv.storage).unwrap();
        let kv_bucket_repository = kv_repository.make_bucket(Some(name)).unwrap();
        let kv_adapter = KVRepositoryKeyValueAdapter::new(Data::new(kv_bucket_repository));
        TestAdapterData {
            kv_repository,
            kv_adapter,
        }
    }

    #[test]
    fn test_get_del_set() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::adapters::key_value::kv::tests::test_get_del_set
        let key = "app_adapters_key_value_kv_tests_test_get_del_set";
        let data = make_adapter(key);
        data.kv_adapter.repository.get_ref().clear().unwrap();

        assert_eq!(data.kv_adapter.get::<u64>(key).unwrap(), None);
        data.kv_adapter.set::<u64>(key, 34).unwrap();
        assert_eq!(data.kv_adapter.get::<u64>(key).unwrap(), Some(34));
        assert_eq!(data.kv_adapter.get::<u64>(key).unwrap(), Some(34));
        data.kv_adapter.set::<u64>(key, 22).unwrap();
        assert_eq!(data.kv_adapter.get::<u64>(key).unwrap(), Some(22));
        data.kv_adapter
            .set::<String>(key, "Hello".to_string())
            .unwrap();
        assert_eq!(
            data.kv_adapter.get::<String>(key).unwrap(),
            Some("Hello".to_string())
        );
        assert!(data.kv_adapter.get::<u64>(key).is_err());
        data.kv_adapter.set::<u64>(key, 22).unwrap();
        assert_eq!(data.kv_adapter.get::<u64>(key).unwrap(), Some(22));
        data.kv_adapter.del(key).unwrap();
        assert_eq!(data.kv_adapter.get::<u64>(key).unwrap(), None);
        assert_eq!(data.kv_adapter.get_del::<u64>(key).unwrap(), None);
        data.kv_adapter.set::<u64>(key, 22).unwrap();
        assert_eq!(data.kv_adapter.get::<u64>(key).unwrap(), Some(22));
        assert_eq!(data.kv_adapter.get_del::<u64>(key).unwrap(), Some(22));
        assert_eq!(data.kv_adapter.get_del::<u64>(key).unwrap(), None);
        assert_eq!(data.kv_adapter.get::<u64>(key).unwrap(), None);

        data.kv_adapter.repository.get_ref().clear().unwrap();
    }

    #[test]
    fn test_set_and_set_ex() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::adapters::key_value::kv::tests::test_set_and_set_ex
        let key = "app_adapters_key_value_kv_tests_test_set_and_set_ex";
        let data = make_adapter(key);
        data.kv_adapter.repository.get_ref().clear().unwrap();

        data.kv_adapter.set_ex::<u64>(key, 22, 100).unwrap();
        assert_eq!(data.kv_adapter.get::<u64>(key).unwrap(), Some(22));
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), 100);
        data.kv_adapter.set::<u64>(key, 22).unwrap();
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), u64::MAX);

        data.kv_adapter.repository.get_ref().clear().unwrap();
    }

    #[test]
    fn test_incr() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::adapters::key_value::kv::tests::test_incr
        let key = "app_adapters_key_value_kv_tests_test_incr";
        let data = make_adapter(key);
        data.kv_adapter.repository.get_ref().clear().unwrap();

        assert_eq!(data.kv_adapter.incr(key, 5).unwrap(), 5);
        assert_eq!(data.kv_adapter.get::<i64>(key).unwrap(), Some(5));
        assert_eq!(data.kv_adapter.incr(key, 5).unwrap(), 10);
        assert_eq!(data.kv_adapter.get::<i64>(key).unwrap(), Some(10));
        assert_eq!(data.kv_adapter.incr(key, -2).unwrap(), 8);
        assert_eq!(data.kv_adapter.incr(key, -20).unwrap(), -12);
        assert_eq!(data.kv_adapter.incr(key, 12).unwrap(), 0);

        data.kv_adapter.repository.get_ref().clear().unwrap();
    }

    #[test]
    fn test_incr_ex() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::adapters::key_value::kv::tests::test_incr_ex
        let key = "app_adapters_key_value_kv_tests_test_incr_ex";
        let data = make_adapter(key);
        data.kv_adapter.repository.get_ref().clear().unwrap();

        data.kv_adapter.set_ex::<i64>(key, 5, 50).unwrap();
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), 50);

        assert_eq!(data.kv_adapter.incr(key, 5).unwrap(), 10);
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), 50);
        assert_eq!(data.kv_adapter.get::<i64>(key).unwrap(), Some(10));

        data.kv_adapter.repository.get_ref().clear().unwrap();
    }

    #[test]
    fn test_get_ex() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::adapters::key_value::kv::tests::test_get_ex
        let key = "app_adapters_key_value_kv_tests_test_get_ex";
        let data = make_adapter(key);
        data.kv_adapter.repository.get_ref().clear().unwrap();

        assert_eq!(data.kv_adapter.get_ex::<u64>(key, 100).unwrap(), None);
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), u64::MAX);
        data.kv_adapter.set::<u64>(key, 5).unwrap();
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), u64::MAX);
        assert_eq!(data.kv_adapter.get_ex::<u64>(key, 100).unwrap(), Some(5));
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), 100);

        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), 99);

        assert_eq!(data.kv_adapter.get_ex::<u64>(key, 1).unwrap(), Some(5));
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(data.kv_adapter.get::<u64>(key).unwrap(), None);
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), u64::MAX);

        data.kv_adapter.repository.get_ref().clear().unwrap();
    }

    #[test]
    fn test_set_ex() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::adapters::key_value::kv::tests::test_set_ex
        let key = "app_adapters_key_value_kv_tests_test_set_ex";
        let data = make_adapter(key);
        data.kv_adapter.repository.get_ref().clear().unwrap();

        data.kv_adapter.set_ex::<u64>(key, 5, 50).unwrap();
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), 50);

        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), 49);
        data.kv_adapter.set_ex::<u64>(key, 5, 1).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(data.kv_adapter.get::<u64>(key).unwrap(), None);
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), u64::MAX);

        data.kv_adapter.repository.get_ref().clear().unwrap();
    }

    #[test]
    fn test_expire() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::adapters::key_value::kv::tests::test_expire
        let key = "app_adapters_key_value_kv_tests_test_expire";
        let data = make_adapter(key);
        data.kv_adapter.repository.get_ref().clear().unwrap();

        data.kv_adapter.set_ex::<u64>(key, 5, 100).unwrap();
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), 100);
        data.kv_adapter.expire(key, 50).unwrap();
        assert_eq!(data.kv_adapter.ttl(key).unwrap(), 50);

        data.kv_adapter.repository.get_ref().clear().unwrap();
    }

    #[test]
    fn test_clean_expired_values() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::adapters::key_value::kv::tests::test_clean_expired_values
        let key = "app_adapters_key_value_kv_tests_test_clean_expired_values";
        let mut data = make_adapter(key);
        data.kv_adapter.repository.get_ref().clear().unwrap();

        data.kv_adapter.set_ex::<u64>(key, 5, 1).unwrap();
        data.kv_adapter.clean_expired_values().unwrap();
        assert!(data.kv_adapter.repository.get_ref().get(key.as_bytes()).unwrap().is_some());
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(data.kv_adapter.repository.get_ref().get(key.as_bytes()).unwrap().is_some());
        data.kv_adapter.clean_expired_values().unwrap();
        assert!(data.kv_adapter.repository.get_ref().get(key.as_bytes()).unwrap().is_none());

        data.kv_adapter.repository.get_ref().clear().unwrap();
    }
}
