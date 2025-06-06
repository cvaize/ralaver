pub mod role;
pub mod user;

pub use self::role::*;
pub use self::user::*;
use r2d2_mysql::mysql::{Row, Value};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;
use strum::VariantNames;

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

pub trait MysqlAllColumnEnum {
    fn mysql_all_select_columns() -> String {
        "".to_string()
    }
    fn mysql_all_insert_columns() -> String {
        "".to_string()
    }
    fn mysql_all_update_columns() -> String {
        "".to_string()
    }
}

pub trait MysqlColumnEnum {
    fn mysql_select_columns(&self) -> String {
        "".to_string()
    }
    fn mysql_insert_columns(&self) -> String {
        "".to_string()
    }
    fn mysql_update_columns(&self) -> String {
        "".to_string()
    }
}

pub trait ToMysqlDto<T>
where
    T: Display + VariantNames + strum::IntoEnumIterator,
{
    #[allow(unused_variables)]
    fn push_mysql_param_to_vec(&self, column: &T, params: &mut Vec<(String, Value)>) {}
    fn push_mysql_params_to_vec(
        &self,
        columns: &Option<Vec<T>>,
        params: &mut Vec<(String, Value)>,
    ) {
        if let Some(columns) = columns {
            for column in columns.iter() {
                self.push_mysql_param_to_vec(&column, params);
            }
        } else {
            for column in T::iter() {
                self.push_mysql_param_to_vec(&column, params);
            }
        }
    }
    fn push_all_mysql_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        for column in T::iter() {
            self.push_mysql_param_to_vec(&column, params);
        }
    }
}

pub trait FromMysqlDto {
    fn take_from_mysql_row(row: &mut Row) -> Result<Self, FromDbRowError>
    where
        Self: Sized;
}

impl<T: Display + VariantNames + MysqlColumnEnum> MysqlAllColumnEnum for T {
    fn mysql_all_select_columns() -> String {
        T::VARIANTS.join(",").to_string()
    }
    fn mysql_all_insert_columns() -> String {
        let columns = Self::mysql_all_select_columns();
        let set = Self::mysql_all_update_columns();

        let mut s = "(".to_string();
        s.push_str(&columns);
        s.push_str(") VALUES (");
        s.push_str(&set);
        s.push_str(")");
        s
    }
    fn mysql_all_update_columns() -> String {
        let t: Vec<String> = T::VARIANTS
            .iter()
            .map(|t| {
                let mut s = t.to_string();
                s.push_str("=:");
                s.push_str(t);
                s
            })
            .collect();
        t.join(",").to_string()
    }
}

impl<T> MysqlColumnEnum for Option<Vec<T>>
where
    T: Display + VariantNames + MysqlAllColumnEnum,
{
    fn mysql_select_columns(&self) -> String {
        // id,email,locale,surname,name,patronymic,is_super_admin
        if let Some(vec) = self {
            if vec.len() > 0 {
                let t: Vec<String> = vec.iter().map(|t| t.to_string()).collect();
                return t.join(",").to_string();
            }
        }
        T::mysql_all_select_columns()
    }
    fn mysql_insert_columns(&self) -> String {
        // (email, locale, surname, name, patronymic) VALUES (:email, :locale, :surname, :name, :patronymic)
        let columns = self.mysql_select_columns();
        let set = self.mysql_update_columns();

        let mut s = "(".to_string();
        s.push_str(&columns);
        s.push_str(") VALUES (");
        s.push_str(&set);
        s.push_str(")");
        s
    }
    fn mysql_update_columns(&self) -> String {
        // email=:email, locale=:locale, surname=:surname, name=:name, patronymic=:patronymic
        if let Some(vec) = self {
            if vec.len() > 0 {
                let t: Vec<String> = vec
                    .iter()
                    .map(|t| {
                        let mut s = t.to_string();
                        s.push_str("=:");
                        s.push_str(t.to_string().as_str());
                        s
                    })
                    .collect();
                return t.join(",").to_string();
            }
        }

        T::mysql_all_update_columns()
    }
}

pub fn make_pagination_mysql_query(
    table: &str,
    columns: &str,
    where_: &str,
    order_: &str,
) -> String {
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
