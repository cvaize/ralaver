use strum_macros::{Display, EnumString};

pub mod mysql;
pub mod mysql2;
pub mod redis;
pub mod smtp;

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum ConnectionError {
    CreatePoolFail,
    CreateClientFail,
}
