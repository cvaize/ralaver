use strum_macros::{Display, EnumString};

pub mod mysql;
pub mod redis;
pub mod smtp;
pub mod sqlite_memory;

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum ConnectionError {
    CreatePoolFail,
    CreateClientFail,
}
