use crate::app::connections::ConnectionError;
use crate::config::MysqlDbConfig;
use diesel::mysql::Mysql;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::r2d2::ConnectionManager;
use diesel::sql_types::BigInt;
use diesel::MysqlConnection;
use r2d2::{Pool, PooledConnection};
use serde_derive::{Deserialize, Serialize};

pub type DieselMysqlPool = Pool<ConnectionManager<MysqlConnection>>;
pub type DieselMysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

pub fn get_connection_pool(config: &MysqlDbConfig) -> Result<DieselMysqlPool, ConnectionError> {
    log::info!("Connecting to MySQL database.");
    let database_url = config.url.to_owned();
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    Pool::builder().build(manager).map_err(|e| {
        log::error!("ConnectionError::CreatePoolFail - {e}");
        ConnectionError::CreatePoolFail
    })
}