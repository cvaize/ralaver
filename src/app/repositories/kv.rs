use crate::AppError;
use kv::{Bucket, Config, Error, Item, Iter, Raw, Store, Value};

pub struct KVRepository<'a> {
    store: Store,
    bucket: KVBucketRepository<'a>,
}

fn make_bucket<'a>(store: &Store, name: Option<&str>) -> Result<Bucket<'a, Raw, Raw>, AppError> {
    store.bucket::<Raw, Raw>(name).map_err(|e| {
        log::error!("KVRepository::make_bucket - {e}");
        AppError(Some(e.to_string()))
    })
}

impl<'a> KVRepository<'a> {
    pub fn new(storage: &str) -> Result<Self, AppError> {
        let store = Store::new(Config::new(storage)).map_err(|e| {
            log::error!("KVRepository::new - {e}");
            AppError(Some(e.to_string()))
        })?;
        let bucket = KVBucketRepository::new(make_bucket(&store, None)?);
        Ok(Self { store, bucket })
    }

    pub fn make_bucket(&self, name: Option<&str>) -> Result<Bucket<'a, Raw, Raw>, AppError> {
        make_bucket(&self.store, name)
    }

    pub fn contains(&self, key: &[u8]) -> Result<bool, AppError> {
        self.bucket.contains(key)
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, AppError> {
        self.bucket.get(key)
    }

    pub fn set(&self, key: &[u8], value: &[u8]) -> Result<Option<Vec<u8>>, AppError> {
        self.bucket.set(key, value)
    }

    pub fn remove(&self, key: &[u8]) -> Result<Option<Vec<u8>>, AppError> {
        self.bucket.remove(key)
    }

    pub fn iter(&self) -> KVIterRepository {
        self.bucket.iter()
    }
}

pub struct KVBucketRepository<'a> {
    bucket: Bucket<'a, Raw, Raw>,
}

impl<'a> KVBucketRepository<'a> {
    pub fn new(bucket: Bucket<'a, Raw, Raw>) -> Self {
        Self { bucket }
    }

    pub fn parse_value(&self, value: Option<Raw>) -> Result<Option<Vec<u8>>, AppError> {
        if let Some(value) = value {
            Ok(Some(value.to_vec()))
        } else {
            Ok(None)
        }
    }

    pub fn contains(&self, key: &[u8]) -> Result<bool, AppError> {
        let key = Raw::from(key);
        let value = self.bucket.contains(&key).map_err(|e| {
            log::error!("KVRepository::contains - {e}");
            AppError(Some(e.to_string()))
        })?;
        Ok(value)
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, AppError> {
        let key = Raw::from(key);
        let value = self.bucket.get(&key).map_err(|e| {
            log::error!("KVRepository::get - {e}");
            AppError(Some(e.to_string()))
        })?;
        self.parse_value(value)
    }

    pub fn set(&self, key: &[u8], value: &[u8]) -> Result<Option<Vec<u8>>, AppError> {
        let key = Raw::from(key);
        let value = Raw::from(value);
        let old_value = self.bucket.set(&key, &value).map_err(|e| {
            log::error!("KVRepository::set - {e}");
            AppError(Some(e.to_string()))
        })?;
        self.parse_value(old_value)
    }

    pub fn remove(&self, key: &[u8]) -> Result<Option<Vec<u8>>, AppError> {
        let key = Raw::from(key);
        let old_value = self.bucket.remove(&key).map_err(|e| {
            log::error!("KVRepository::remove - {e}");
            AppError(Some(e.to_string()))
        })?;
        self.parse_value(old_value)
    }

    pub fn iter(&self) -> KVIterRepository {
        KVIterRepository {
            iter: self.bucket.iter(),
        }
    }
}

pub struct KVIterRepository {
    iter: Iter<Raw, Raw>,
}

impl Iterator for KVIterRepository {
    type Item = Result<(Vec<u8>, Vec<u8>), AppError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some(Err(e)) => Some(Err(AppError(Some(e.to_string())))),
            Some(Ok(item)) => {
                let key: Result<Raw, Error> = item.key();
                let value: Result<Raw, Error> = item.value();

                if let Err(error) = key {
                    return Some(Err(AppError(Some(error.to_string()))));
                }

                if let Err(error) = value {
                    return Some(Err(AppError(Some(error.to_string()))));
                }

                let key: Raw = key.unwrap();
                let value: Raw = value.unwrap();

                Some(Ok((key.to_vec(), value.to_vec())))
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::preparation;
//     use test::Bencher;
//
//     // // #[bench]
//     // // fn bench_kv(b: &mut Bencher) {
//     // //     // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo bench -- --nocapture --exact app::repositories::kv::tests::bench_kv
//     // #[test]
//     // fn test_kv() {
//     //     // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::kv::tests::test_kv
//     //     // При реализации expires можно при получении данных проверять expires и если он истёк, то удалять старое значение и отдавать в ответе None.
//     //     let storage = "./storage/kv_db";
//     //     let store = Store::new(Config::new(storage)).unwrap();
//     //     let bucket = store.bucket::<Raw, Raw>(None).unwrap();
//     //
//     //     let key = Raw::from("test_key_1".as_bytes());
//     //     let value = Raw::from("test_value_1".as_bytes());
//     //
//     //     let result = bucket.set(&key, &value).unwrap();
//     //     if let Some(result) = result {
//     //         dbg!(String::from_utf8(result.to_vec()).unwrap());
//     //     } else {
//     //         dbg!("None");
//     //     }
//     //
//     //     let key = Raw::from("test_key_2".as_bytes());
//     //     let value = Raw::from("test_value_2".as_bytes());
//     //
//     //     let result = bucket.set(&key, &value).unwrap();
//     //     if let Some(result) = result {
//     //         dbg!(String::from_utf8(result.to_vec()).unwrap());
//     //     } else {
//     //         dbg!("None");
//     //     }
//     //
//     //     let key = Raw::from("test_key_2".as_bytes());
//     //     let value = Raw::from("test_value_3".as_bytes());
//     //
//     //     let result = bucket.set(&key, &value).unwrap();
//     //     if let Some(result) = result {
//     //         dbg!(String::from_utf8(result.to_vec()).unwrap());
//     //     } else {
//     //         dbg!("None");
//     //     }
//     //
//     //     // b.iter(|| {
//     //     //     // 190.75 ns/iter (+/- 6.52)
//     //     //     // let _ = bucket.get(&key).unwrap().unwrap();
//     //     //     // 15.06 ns/iter (+/- 0.29)
//     //     //     let _ = store.bucket::<Raw, Raw>(None).unwrap();
//     //     // });
//     // }
// }
