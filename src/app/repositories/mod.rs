mod file;
mod role;
mod user;

pub use self::file::*;
pub use self::role::*;
pub use self::user::*;
use r2d2_mysql::mysql::{Row, Value};
use std::fmt::Display;
use r2d2_mysql::mysql::prelude::FromValue;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use strum::VariantNames;
use crate::UserColumn;

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

#[derive(Debug)]
pub struct PaginateParams<F, S> {
    pub page: i64,
    pub per_page: i64,
    pub filters: Vec<F>,
    pub sort: Option<S>,
}

pub struct ToDbValueError;
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
        // id,email,locale,surname,name,patronymic,is_super_admin
        T::VARIANTS.join(",").to_string()
    }
    fn mysql_all_insert_columns() -> String {
        // (email, locale, surname, name, patronymic) VALUES (:email, :locale, :surname, :name, :patronymic)
        let columns = Self::mysql_all_select_columns();
        let values = T::VARIANTS.join(",:").to_string();

        let mut s = "(".to_string();
        s.push_str(&columns);
        s.push_str(") VALUES (:");
        s.push_str(&values);
        s.push_str(")");
        s
    }
    fn mysql_all_update_columns() -> String {
        // email=:email, locale=:locale, surname=:surname, name=:name, patronymic=:patronymic
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
        if let Some(vec) = self {
            if vec.len() > 0 {
                let columns = self.mysql_select_columns();

                let t: Vec<String> = vec.iter().map(|t| t.to_string()).collect();
                let values = t.join(",:").to_string();

                let mut s = "(".to_string();
                s.push_str(&columns);
                s.push_str(") VALUES (:");
                s.push_str(&values);
                s.push_str(")");
                return s;
            }
        }

        T::mysql_all_insert_columns()
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
    sql.push_str(" ");
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

pub fn take_from_mysql_row<T: FromValue>(row: &mut Row, name: &str) -> Result<T, FromDbRowError>{
    if let Some(val) = row.take_opt::<T, &str>(name) {
        return val.map_err(|_| FromDbRowError)
    }
    Err(FromDbRowError)
}

pub fn take_json_from_mysql_row<T: DeserializeOwned>(row: &mut Row, name: &str) -> Result<T, FromDbRowError>{
    if let Some(val) = row.take_opt::<String, &str>(name) {
        if let Ok(val) = val {
            if val.len() == 0 {
                return Err(FromDbRowError);
            }
            let val: serde_json::Result<T> = serde_json::from_str(&val);
            if let Ok(val) = val {
                return Ok(val);
            }
            return Err(FromDbRowError);
        }

        return Err(FromDbRowError);
    }
    Err(FromDbRowError)
}

pub fn option_take_json_from_mysql_row<T: DeserializeOwned>(row: &mut Row, name: &str) -> Option<T>{
    if let Ok(v) = take_json_from_mysql_row(row, name) {
        Some(v)
    } else {
        None
    }
}

pub fn to_json_string_for_mysql<T: Serialize>(val: &T) -> Result<String, ToDbValueError> {
    let val: serde_json::Result<String> = serde_json::to_string(&val);
    if let Ok(val) = val {
        Ok(val)
    } else {
        Err(ToDbValueError)
    }
}

pub fn option_to_json_string_for_mysql<T: Serialize>(val: &Option<T>) -> Option<String> {
    if let Some(val) = val{
        if let Ok(v) = to_json_string_for_mysql(val) {
            Some(v)
        } else {
            None
        }
    } else {
        None
    }
}