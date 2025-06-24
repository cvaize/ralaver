use crate::config::MysqlDbConfig;
use r2d2::{Pool, PooledConnection};
use crate::errors::AppError;
use mysql::{error::Error, Conn, Opts, OptsBuilder};

/// An [`r2d2`] connection manager for [`mysql`] connections.
#[derive(Clone, Debug)]
pub struct MySqlConnectionManager {
    params: Opts,
}

impl MySqlConnectionManager {
    /// Constructs a new MySQL connection manager from `params`.
    pub fn new(params: OptsBuilder) -> MySqlConnectionManager {
        MySqlConnectionManager {
            params: Opts::from(params),
        }
    }
}

impl r2d2::ManageConnection for MySqlConnectionManager {
    type Connection = Conn;
    type Error = Error;

    fn connect(&self) -> Result<Conn, Error> {
        Conn::new(self.params.clone())
    }

    fn is_valid(&self, conn: &mut Conn) -> Result<(), Error> {
        conn.ping()
    }

    fn has_broken(&self, conn: &mut Conn) -> bool {
        self.is_valid(conn).is_err()
    }
}

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