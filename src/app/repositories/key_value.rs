use strum_macros::{Display, EnumString};

macro_rules! key_value_impl {
    () => {
        fn get<K, V>(&mut self, key: K) -> Result<Option<V>, KeyValueRepositoryError>;
        fn get_ex<K, V>(
            &mut self,
            key: K,
            seconds: u64,
        ) -> Result<Option<V>, KeyValueRepositoryError>;
        fn get_del<K, V>(&mut self, key: K) -> Result<Option<V>, KeyValueRepositoryError>;
        fn set<K, V>(&mut self, key: K, value: V) -> Result<(), KeyValueRepositoryError>;
        fn set_ex<K, V>(
            &mut self,
            key: K,
            value: V,
            seconds: u64,
        ) -> Result<(), KeyValueRepositoryError>;
        fn expire<K>(&mut self, key: K, seconds: i64) -> Result<(), KeyValueRepositoryError>;
        fn del<K>(&mut self, key: K) -> Result<(), KeyValueRepositoryError>;
        fn incr<K, D, V>(&mut self, key: K, delta: D) -> Result<V, KeyValueRepositoryError>;
        fn ttl<K, V>(&mut self, key: K) -> Result<V, KeyValueRepositoryError>;
    };
}

pub trait KeyValueConnection {
    key_value_impl!();
}

pub trait KeyValueRepository {
    fn get_connection(&self) -> Result<impl KeyValueConnection, KeyValueRepositoryError>;
    key_value_impl!();
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum KeyValueRepositoryError {
    ConnectFail,
    Fail,
}
