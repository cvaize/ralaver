use crate::config::MysqlDbConfig;
use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;
use diesel::mysql::Mysql;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::sql_types::BigInt;
use r2d2::{Pool, PooledConnection};
use serde_derive::{Deserialize, Serialize};
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

pub trait Paginate: Sized {
    fn paginate(self, page: i64, per_page: i64) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, page: i64, per_page: i64) -> Paginated<Self> {
        Paginated {
            query: self,
            per_page,
            page,
            offset: (page - 1) * per_page,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    query: T,
    page: i64,
    per_page: i64,
    offset: i64,
}

impl<T> Paginated<T> {

    pub fn load_and_count_pages<'a, U>(self, conn: &mut MysqlConnection) -> QueryResult<PaginationResult<U>>
    where
        Self: LoadQuery<'a, MysqlConnection, (U, i64)>,
    {
        let page: i64 = self.page;
        let per_page: i64 = self.per_page;
        let results = self.load::<(U, i64)>(conn)?;
        let total_records: i64 = results.first().map(|x| x.1).unwrap_or(0);
        let records: Vec<U> = results.into_iter().map(|x| x.0).collect();
        let total_pages: i64 = (total_records as f64 / per_page as f64).ceil() as i64;
        Ok(PaginationResult{page, per_page, total_pages, total_records, records})
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<MysqlConnection> for Paginated<T> {}

impl<T> QueryFragment<Mysql> for Paginated<T>
where
    T: QueryFragment<Mysql>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Mysql>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.per_page)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.offset)?;
        Ok(())
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct PaginationResult<U> {
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
    pub total_records: i64,
    pub records: Vec<U>
}