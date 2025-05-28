mod user;

use r2d2_mysql::mysql::{Params, Row};
pub use self::user::*;
use serde_derive::{Deserialize, Serialize};

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

pub struct FromDbRowError;

pub trait ToMysqlDto {
    fn to_db_params(&self) -> Params;
    fn db_select_columns() -> String {
        "".to_string()
    }
    fn db_insert_columns() -> String {
        "".to_string()
    }
    fn db_update_columns() -> String {
        "".to_string()
    }
}

pub trait FromMysqlDto {
    fn take_from_db_row(row: &mut Row) -> Result<Self, FromDbRowError> where Self: Sized;
}

pub fn make_pagination_mysql_query(table: &str, columns: &str, where_: &str, order_: &str) -> String {
    let mut sql = "SELECT ".to_string();
    sql.push_str(columns);
    sql.push_str(", COUNT(*) OVER () as total_records FROM ");
    sql.push_str(table);
    if where_.len() > 0 {
        sql.push_str(" WHERE ");
        sql.push_str(where_);
    }
    if order_.len() > 0 {
        sql.push_str(" ORDER BY ");
        sql.push_str(order_);
    }
    sql.push_str(" LIMIT :per_page OFFSET :offset");
    sql
}

pub fn make_select_mysql_query(table: &str, columns: &str, where_: &str, order_: &str) -> String {
    let mut sql = "SELECT ".to_string();
    sql.push_str(columns);
    sql.push_str(" FROM ");
    sql.push_str(table);
    if where_.len() > 0 {
        sql.push_str(" WHERE ");
        sql.push_str(where_);
    }
    if order_.len() > 0 {
        sql.push_str(" ORDER BY ");
        sql.push_str(order_);
    }
    sql
}

pub fn make_is_exists_mysql_query(table: &str, where_: &str) -> String {
    let mut sql = "SELECT EXISTS(SELECT 1 FROM ".to_string();
    sql.push_str(table);
    sql.push_str(" WHERE ");
    sql.push_str(where_);
    sql.push_str(" LIMIT 1) as is_exists");
    sql
}

pub fn make_insert_mysql_query(table: &str, columns_: &str) -> String {
    let mut sql = "INSERT INTO ".to_string();
    sql.push_str(table);
    sql.push_str(columns_);
    sql
}

pub fn make_update_mysql_query(table: &str, set_: &str, where_: &str) -> String {
    let mut sql = "UPDATE ".to_string();
    sql.push_str(table);
    sql.push_str(" SET ");
    sql.push_str(set_);
    sql.push_str(" WHERE ");
    sql.push_str(where_);
    sql
}

pub fn make_delete_mysql_query(table: &str, where_: &str) -> String {
    let mut sql = "DELETE FROM ".to_string();
    sql.push_str(table);
    sql.push_str(" WHERE ");
    sql.push_str(where_);
    sql
}