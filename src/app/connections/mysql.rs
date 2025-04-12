use crate::{Config, LogService};
use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;
use r2d2::Pool;
use strum_macros::{Display, EnumString};

pub type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum MysqlConnectionError {
    CreatePoolFail,
}

pub fn get_connection_pool(
    config: &Config,
    log_service: &LogService,
) -> Result<MysqlPool, MysqlConnectionError> {
    log_service.info("Connecting to MySQL database.");
    let database_url = config.db.mysql.url.to_owned();
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    Pool::builder().build(manager).map_err(|e| {
        log_service.error(format!("MysqlConnectionError::CreatePoolFail - {:}", &e).as_str());
        MysqlConnectionError::CreatePoolFail
    })
    // .expect("Failed to create connection Pool.")
}
