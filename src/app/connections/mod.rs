use strum_macros::{Display, EnumString};

pub mod diesel_mysql;
pub mod mysql;
pub mod redis;
pub mod smtp;

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum ConnectionError {
    CreatePoolFail,
    CreateClientFail,
}
