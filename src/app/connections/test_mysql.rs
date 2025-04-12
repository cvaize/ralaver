use crate::{Config, LogService};
use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;
use r2d2::Pool;
use strum_macros::{Display, EnumString};

pub type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum TestMysqlConnectionError {
    CreatePoolFail,
}

pub fn get_connection_pool(
    config: &Config,
    log_service: &LogService,
) -> Result<MysqlPool, TestMysqlConnectionError> {
    log_service.info("Connecting to test MySQL database.");
    let database_url = config.db.test_mysql.url.to_owned();
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    Pool::builder().build(manager).map_err(|e| {
        log_service.error(format!("TestMysqlConnectionError::CreatePoolFail - {:}", &e).as_str());
        TestMysqlConnectionError::CreatePoolFail
    })
    // .expect("Failed to create connection Pool.")
}
