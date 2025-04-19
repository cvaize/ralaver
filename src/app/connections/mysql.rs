use crate::config::MysqlDbConfig;
use crate::Log;
use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;
use r2d2::{Pool, PooledConnection};
use strum_macros::{Display, EnumString};

pub type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;
pub type MysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum MysqlConnectionError {
    CreatePoolFail,
}

pub fn get_connection_pool(
    config: &MysqlDbConfig,
) -> Result<MysqlPool, MysqlConnectionError> {
    Log::info("Connecting to MySQL database.");
    let database_url = config.url.to_owned();
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    Pool::builder().build(manager).map_err(|e| {
        Log::error(format!("MysqlConnectionError::CreatePoolFail - {:}", &e).as_str());
        MysqlConnectionError::CreatePoolFail
    })
}