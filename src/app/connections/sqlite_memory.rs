use diesel::r2d2::ConnectionManager;
use diesel::SqliteConnection;
use r2d2::{Pool, PooledConnection};
use crate::app::connections::ConnectionError;

#[allow(dead_code)]
pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
#[allow(dead_code)]
pub type SqlitePooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

#[allow(dead_code)]
pub fn get_connection_pool() -> Result<SqlitePool, ConnectionError> {
    log::info!("{}","Connecting to memory SQLite database.");
    let manager = ConnectionManager::<SqliteConnection>::new(":memory:");

    Pool::builder().build(manager).map_err(|e| {
        log::error!("{}",format!("ConnectionError::CreatePoolFail - {:}", &e).as_str());
        ConnectionError::CreatePoolFail
    })
}