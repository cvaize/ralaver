use crate::config::MysqlDbConfig;
use r2d2_mysql::mysql::{OptsBuilder, Opts};
use r2d2::{Pool, PooledConnection};
use r2d2_mysql::MySqlConnectionManager;
use crate::errors::AppError;

pub type MysqlPool = Pool<MySqlConnectionManager>;
pub type MysqlPooledConnection = PooledConnection<MySqlConnectionManager>;

pub fn get_connection_pool(config: &MysqlDbConfig) -> Result<MysqlPool, AppError> {
    log::info!("Connecting to MySQL database.");
    // let o = OptsBuilder::new()
    // .db_name(Some(&config.database))
    // .user(Some(&config.user))
    // .pass(Some(&config.password));
    let opts = Opts::from_url(&config.url).unwrap();
    let builder = OptsBuilder::from_opts(opts);

    let manager = MySqlConnectionManager::new(builder);

    Pool::builder().build(manager).map_err(|e| {
        log::error!("ConnectionError::CreatePoolFail - {}", &e);
        AppError(Some(e.to_string()))
    })
}