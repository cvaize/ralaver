use crate::app::connections::ConnectionError;
use crate::config::MysqlDbConfig;
use r2d2_mysql::mysql::{OptsBuilder, QueryResult, from_row, Opts};
use r2d2::Pool;
use r2d2_mysql::MySqlConnectionManager;

pub type MysqlPool2 = Pool<MySqlConnectionManager>;

pub fn get_connection_pool(config: &MysqlDbConfig) -> Result<MysqlPool2, ConnectionError> {
    log::info!("Connecting to MySQL database.");
    // let o = OptsBuilder::new()
    // .db_name(Some(&config.database))
    // .user(Some(&config.user))
    // .pass(Some(&config.password));
    let opts = Opts::from_url(&config.url).unwrap();
    let builder = OptsBuilder::from_opts(opts);

    let manager = MySqlConnectionManager::new(builder);

    Pool::builder().build(manager).map_err(|e| {
        log::error!("ConnectionError::CreatePoolFail - {e}");
        ConnectionError::CreatePoolFail
    })
}
