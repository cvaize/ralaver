use crate::AppError;
use crate::helpers::{BytesKey, BytesValue};

pub struct RedisKeyValueAdapter {
    // conn: RedisAdapterConnection
}

// impl KeyValueConnectionAdapter for RedisAdapterKeyValueConnectionAdapter {
//     fn get<V: BytesValue>(&mut self, key: &str) -> Result<Option<V>, AppError> {
//         let v: Option<Vec<u8>> = self.conn.get(key)?;
//         match v {
//             Some(v) => Ok(Some(V::value_from_bytes(v)?)),
//             _ => Ok(None)
//         }
//     }
//
//     fn get_ex<V: BytesValue>(&mut self, key: &str, seconds: u64) -> Result<Option<V>, AppError> {
//         let v: Option<Vec<u8>> = self.conn.get_ex(key, seconds)?;
//         match v {
//             Some(v) => Ok(Some(V::value_from_bytes(v)?)),
//             _ => Ok(None)
//         }
//     }
//
//     fn get_del<V: BytesValue>(&mut self, key: &str) -> Result<Option<V>, AppError> {
//         let v: Option<Vec<u8>> = self.conn.get_del(key)?;
//         match v {
//             Some(v) => Ok(Some(V::value_from_bytes(v)?)),
//             _ => Ok(None)
//         }
//     }
//
//     fn set<V: BytesValue>(&mut self, key: &str, value: V) -> Result<(), AppError> {
//         let v: Option<Vec<u8>> = self.conn.set(key, value)?;
//         match v {
//             Some(v) => Ok(Some(V::value_from_bytes(v)?)),
//             _ => Ok(None)
//         }
//     }
//
//     fn set_ex<V: BytesValue>(&mut self, key: &str, value: V, seconds: u64) -> Result<(), AppError> {
//         todo!()
//     }
//
//     fn expire(&mut self, key: &str, seconds: u64) -> Result<(), AppError> {
//         todo!()
//     }
//
//     fn del(&mut self, key: &str) -> Result<(), AppError> {
//         todo!()
//     }
//
//     fn incr(&mut self, key: &str, delta: i64) -> Result<Vec<u8>, AppError> {
//         todo!()
//     }
//
//     fn ttl(&mut self, key: &str) -> Result<u64, AppError> {
//         todo!()
//     }
// }