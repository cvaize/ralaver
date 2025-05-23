mod user;

pub use self::user::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Value<T> {
    Set(T),
    #[default]
    Null,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginationResult<U> {
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
    pub total_records: i64,
    pub records: Vec<U>,
}

impl<U> PaginationResult<U> {
    pub fn new(page: i64, per_page: i64, total_records: i64, records: Vec<U>) -> Self {
        Self {
            page,
            per_page,
            total_pages: (total_records as f64 / per_page as f64).ceil() as i64,
            total_records,
            records,
        }
    }
}


pub fn make_pagination_mysql_query(columns: &str, table: &str, where_: &str) -> String {
    let mut sql = "SELECT ".to_string();
    sql.push_str(columns);
    sql.push_str(", COUNT(*) OVER () as total_records FROM ");
    sql.push_str(table);
    if where_.len() > 0 {
        sql.push_str(" WHERE ");
        sql.push_str(where_);
    }
    sql.push_str(" LIMIT :per_page OFFSET :offset");
    sql
}