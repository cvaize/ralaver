use crate::config::MysqlDbConfig;
use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;
use r2d2::{Pool, PooledConnection};
use crate::app::connections::ConnectionError;

pub type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;
pub type MysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

pub fn get_connection_pool(
    config: &MysqlDbConfig,
) -> Result<MysqlPool, ConnectionError> {
    log::info!("{}","Connecting to MySQL database.");
    let database_url = config.url.to_owned();
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    Pool::builder().build(manager).map_err(|e| {
        log::error!("{}",format!("ConnectionError::CreatePoolFail - {:}", &e).as_str());
        ConnectionError::CreatePoolFail
    })
}