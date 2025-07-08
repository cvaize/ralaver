use crate::helpers::DATE_TIME_FORMAT;
use crate::{AppError, MysqlPool, MysqlPooledConnection, PaginateParams, PaginationResult};
use chrono::NaiveDateTime;
use mysql::prelude::{FromValue, Queryable};
use mysql::{Params, Row, Value};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Display;
use strum::{IntoEnumIterator, VariantNames};

impl<Filter, Sort> MysqlPaginateParams<Filter, Sort> for PaginateParams<Filter, Sort>
where
    Filter: MysqlQueryBuilder,
    Sort: MysqlQueryBuilder,
{
    fn get_page(&self) -> i64 {
        self.page
    }
    fn get_per_page(&self) -> i64 {
        self.per_page
    }
    fn get_filters(&self) -> &Vec<Filter> {
        &self.filters
    }
    fn get_sorts(&self) -> &Vec<Sort> {
        &self.sorts
    }
}

pub trait MysqlRepository<Entity, PaginateParams, EntityColumn, Filter, Sort>
where
    Entity: FromMysqlDto + ToMysqlDto<EntityColumn>,
    PaginateParams: MysqlPaginateParams<Filter, Sort>,
    EntityColumn: IntoEnumIterator
        + Display
        + VariantNames
        + MysqlAllColumnEnum
        + MysqlColumnEnum
        + MysqlIdColumn
        + PartialEq
        + Eq,
    Filter: MysqlQueryBuilder,
    Sort: MysqlQueryBuilder,
{
    fn get_repository_name(&self) -> &str;
    fn get_table(&self) -> &str;
    fn get_db_pool(&self) -> &MysqlPool;
    fn get_is_exists_field(&self) -> &str {
        "is_exists"
    }
    fn get_total_records_field(&self) -> &str {
        "total_records"
    }
    fn get_per_page_field(&self) -> &str {
        "per_page"
    }
    fn get_offset_field(&self) -> &str {
        "offset"
    }
    fn log_error(&self, method_name: &str, original_error_message: String) -> AppError {
        let mut result = self.get_repository_name().to_string();
        result.push_str("::");
        result.push_str(method_name);
        result.push_str(" - ");
        result.push_str(&original_error_message);
        log::error!("{}", result);
        AppError(Some(original_error_message))
    }
    fn connection(&self) -> Result<MysqlPooledConnection, AppError> {
        self.get_db_pool().get().map_err(|e| {
            self.log_error("connection", e.to_string());
            return AppError(Some(e.to_string()));
        })
    }
    fn row_to_entity(&self, row: &mut Row) -> Result<Entity, AppError> {
        Entity::take_from_mysql_row(row).map_err(|e| {
            self.log_error("row_to_entity", e.to_string());
            return e;
        })
    }
    fn try_row_to_entity(&self, row: &mut Option<Row>) -> Result<Option<Entity>, AppError> {
        if let Some(row) = row {
            return Ok(Some(self.row_to_entity(row)?));
        }
        Ok(None)
    }
    fn try_row_is_exists(&self, row: &Option<Row>) -> Result<bool, AppError> {
        if let Some(row) = row {
            return Ok(row.get(self.get_is_exists_field()).unwrap_or(false));
        }
        Ok(false)
    }

    fn all_rows(
        &self,
        filters: Option<&Vec<Filter>>,
        sorts: Option<&Vec<Sort>>,
        columns: &Option<Vec<EntityColumn>>,
    ) -> Result<Vec<Row>, AppError> {
        let table = self.get_table();
        let columns: String = columns.mysql_select_columns();

        let mut mysql_where: String = String::new();
        let mut mysql_order: String = String::new();
        let mut mysql_params: Vec<(String, Value)> = vec![];

        if let Some(filters) = filters {
            let mut is_and = false;
            for filter in filters {
                if is_and {
                    mysql_where.push_str(" AND ")
                }
                filter.push_params_to_vec(&mut mysql_params);
                filter.push_params_to_mysql_query(&mut mysql_where);
                is_and = true;
            }
        }

        if let Some(sorts) = sorts {
            let mut is_and = false;
            for sort in sorts {
                if is_and {
                    mysql_order.push_str(", ")
                }
                sort.push_params_to_vec(&mut mysql_params);
                sort.push_params_to_mysql_query(&mut mysql_order);
                is_and = true;
            }
        }

        let query = make_select_mysql_query(table, &columns, &mysql_where, &mysql_order);
        let mut conn = self.connection()?;

        let mut rows_result: Vec<Row> = Vec::new();
        if mysql_params.is_empty() {
            let rows = conn
                .query_iter(query)
                .map_err(|e| self.log_error("all_rows", e.to_string()))?;
            for mut row in rows.into_iter() {
                if let Ok(row) = row {
                    rows_result.push(row);
                }
            }
        } else {
            let rows = conn
                .exec_iter(query, Params::from(mysql_params))
                .map_err(|e| self.log_error("all_rows", e.to_string()))?;
            for mut row in rows.into_iter() {
                if let Ok(row) = row {
                    rows_result.push(row);
                }
            }
        }

        Ok(rows_result)
    }

    fn all(
        &self,
        filters: Option<&Vec<Filter>>,
        sorts: Option<&Vec<Sort>>,
        columns: &Option<Vec<EntityColumn>>,
    ) -> Result<Vec<Entity>, AppError> {
        let mut records: Vec<Entity> = Vec::new();
        let rows = self.all_rows(filters, sorts, columns)?;
        for mut row in rows {
            records.push(self.row_to_entity(&mut row)?);
        }
        Ok(records)
    }

    fn all_ids(
        &self,
        filters: Option<&Vec<Filter>>,
        sorts: Option<&Vec<Sort>>,
    ) -> Result<Vec<u64>, AppError> {
        let column: EntityColumn = EntityColumn::get_mysql_id_column();
        let column_str: String = column.to_string();
        let column_str: &str = &column_str;
        let columns: Option<Vec<EntityColumn>> = Some(vec![column]);

        let mut ids: Vec<u64> = Vec::new();
        let rows = self.all_rows(filters, sorts, &columns)?;
        for mut row in rows {
            if let Ok(id) = take_from_mysql_row::<u64>(&mut row, column_str) {
                ids.push(id);
            }
        }
        Ok(ids)
    }

    fn first(&self, filters: &Vec<Filter>) -> Result<Option<Entity>, AppError> {
        if filters.is_empty() {
            return Err(AppError(None));
        }
        let table = self.get_table();
        let columns = EntityColumn::mysql_all_select_columns();

        let mut mysql_where: String = String::new();
        let mut mysql_params: Vec<(String, Value)> = Vec::new();

        let mut is_and = false;
        for filter in filters {
            if is_and {
                mysql_where.push_str(" AND ")
            }
            filter.push_params_to_vec(&mut mysql_params);
            filter.push_params_to_mysql_query(&mut mysql_where);
            is_and = true;
        }

        let query = make_select_mysql_query(table, &columns, &mysql_where, "");
        let mut conn = self.connection()?;
        let mut row: Option<Row> = conn
            .exec_first(query, Params::from(mysql_params))
            .map_err(|e| self.log_error("first", e.to_string()))?;

        self.try_row_to_entity(&mut row)
    }

    fn exists(&self, filters: &Vec<Filter>) -> Result<bool, AppError> {
        if filters.is_empty() {
            return Err(AppError(None));
        }
        let table = self.get_table();

        let mut mysql_where: String = String::new();
        let mut mysql_params: Vec<(String, Value)> = Vec::new();

        let mut is_and = false;
        for filter in filters {
            if is_and {
                mysql_where.push_str(" AND ")
            }
            filter.push_params_to_vec(&mut mysql_params);
            filter.push_params_to_mysql_query(&mut mysql_where);
            is_and = true;
        }

        let query = make_is_exists_mysql_query(&table, &mysql_where);
        let mut conn = self.connection()?;
        let row: Option<Row> = conn
            .exec_first(query, Params::from(mysql_params))
            .map_err(|e| self.log_error("exists", e.to_string()))?;

        self.try_row_is_exists(&row)
    }

    fn paginate(&self, params: &PaginateParams) -> Result<PaginationResult<Entity>, AppError> {
        let mut conn = self.connection()?;
        let page = params.get_page();
        let per_page = params.get_per_page();
        let offset = (page - 1) * per_page;

        let per_page_field = self.get_per_page_field();
        let offset_field = self.get_offset_field();

        let mut mysql_where: String = String::new();
        let mut mysql_order: String = String::new();
        let mut mysql_params: Vec<(String, Value)> = vec![
            (String::from(per_page_field), Value::from(per_page)),
            (String::from(offset_field), Value::from(offset)),
        ];

        let mut is_and = false;
        let filters = params.get_filters();
        for filter in filters {
            if is_and {
                mysql_where.push_str(" AND ")
            }
            filter.push_params_to_vec(&mut mysql_params);
            filter.push_params_to_mysql_query(&mut mysql_where);
            is_and = true;
        }

        let mut is_and = false;
        let sorts = params.get_sorts();
        for sort in sorts {
            if is_and {
                mysql_order.push_str(", ")
            }
            sort.push_params_to_vec(&mut mysql_params);
            sort.push_params_to_mysql_query(&mut mysql_order);
            is_and = true;
        }

        let table = self.get_table();
        let columns = EntityColumn::mysql_all_select_columns();
        let query = make_pagination_mysql_query(table, &columns, &mysql_where, &mysql_order);

        let rows = conn
            .exec_iter(&query, Params::from(mysql_params))
            .map_err(|e| self.log_error("paginate", e.to_string()))?;

        let mut records: Vec<Entity> = Vec::new();
        let mut total_records: i64 = 0;
        let total_records_field = self.get_total_records_field();
        for mut row in rows.into_iter() {
            if let Ok(row) = &mut row {
                if total_records == 0 {
                    total_records = row.take(total_records_field).unwrap_or(total_records);
                }
                records.push(self.row_to_entity(row)?);
            }
        }

        Ok(PaginationResult::new(
            page,
            per_page,
            total_records,
            records,
        ))
    }

    fn insert(
        &self,
        data: &Vec<Entity>,
        columns: Option<Vec<EntityColumn>>,
    ) -> Result<(), AppError> {
        let mut conn = self.connection()?;

        let columns_: Option<Vec<EntityColumn>> = if columns.is_none() {
            let id_column = EntityColumn::get_mysql_id_column();
            Some(EntityColumn::iter().filter(|c| c.ne(&id_column)).collect())
        } else {
            columns
        };

        let columns_str = columns_.mysql_insert_columns();
        let table = self.get_table();
        let query = make_insert_mysql_query(table, &columns_str);

        conn.exec_batch(
            query,
            data.iter().map(|entity| {
                let mut params: Vec<(String, Value)> = Vec::new();
                entity.push_mysql_params_to_vec(&columns_, &mut params);
                params
            }),
        )
        .map_err(|e| self.log_error("insert", e.to_string()))?;

        Ok(())
    }

    fn update(
        &self,
        filters: &Vec<Filter>,
        data: &Entity,
        columns: &Option<Vec<EntityColumn>>,
    ) -> Result<(), AppError> {
        if filters.is_empty() {
            return Err(AppError(None));
        }
        let mut conn = self.connection()?;
        let columns_str = columns.mysql_update_columns();

        let table = self.get_table();

        let mut mysql_where: String = String::new();
        let mut mysql_params: Vec<(String, Value)> = Vec::new();

        let mut is_and = false;
        for filter in filters {
            if is_and {
                mysql_where.push_str(" AND ")
            }
            filter.push_params_to_vec(&mut mysql_params);
            filter.push_params_to_mysql_query(&mut mysql_where);
            is_and = true;
        }

        let query = make_update_mysql_query(table, &columns_str, &mysql_where);
        data.push_mysql_params_to_vec(columns, &mut mysql_params);

        conn.exec_drop(query, Params::from(mysql_params))
            .map_err(|e| self.log_error("update", e.to_string()))?;

        Ok(())
    }

    fn delete(&self, filters: &Vec<Filter>) -> Result<(), AppError> {
        if filters.is_empty() {
            return Err(AppError(None));
        }
        let table = self.get_table();

        let mut mysql_where: String = String::new();
        let mut mysql_params: Vec<(String, Value)> = Vec::new();

        let mut is_and = false;
        for filter in filters {
            if is_and {
                mysql_where.push_str(" AND ")
            }
            filter.push_params_to_vec(&mut mysql_params);
            filter.push_params_to_mysql_query(&mut mysql_where);
            is_and = true;
        }

        let mut conn = self.connection()?;
        let query = make_delete_mysql_query(table, &mysql_where);
        conn.exec_drop(query, Params::from(mysql_params))
            .map_err(|e| self.log_error("delete", e.to_string()))?;

        Ok(())
    }
}

pub trait MysqlPaginateParams<F: MysqlQueryBuilder, S: MysqlQueryBuilder> {
    fn get_page(&self) -> i64;
    fn get_per_page(&self) -> i64;
    fn get_filters(&self) -> &Vec<F>;
    fn get_sorts(&self) -> &Vec<S>;
}

pub trait MysqlQueryBuilder {
    fn push_params_to_mysql_query(&self, query: &mut String);
    fn push_params_to_vec(&self, params: &mut Vec<(String, Value)>);
}

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

pub trait MysqlIdColumn {
    fn get_mysql_id_column() -> Self;
}

pub trait ToMysqlDto<EntityColumn>
where
    EntityColumn: Display + VariantNames + IntoEnumIterator,
{
    #[allow(unused_variables)]
    fn push_mysql_param_to_vec(&self, column: &EntityColumn, params: &mut Vec<(String, Value)>) {}
    fn push_mysql_params_to_vec(
        &self,
        columns: &Option<Vec<EntityColumn>>,
        params: &mut Vec<(String, Value)>,
    ) {
        if let Some(columns) = columns {
            for column in columns.iter() {
                self.push_mysql_param_to_vec(&column, params);
            }
        } else {
            for column in EntityColumn::iter() {
                self.push_mysql_param_to_vec(&column, params);
            }
        }
    }
    fn push_all_mysql_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        for column in EntityColumn::iter() {
            self.push_mysql_param_to_vec(&column, params);
        }
    }
    fn get_id(&self) -> u64;
}

pub trait FromMysqlDto {
    fn take_from_mysql_row(row: &mut Row) -> Result<Self, AppError>
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

pub fn take_from_mysql_row<T: FromValue>(row: &mut Row, name: &str) -> Result<T, AppError> {
    if let Some(val) = row.take_opt::<T, &str>(name) {
        return match val {
            Ok(val) => Ok(val),
            Err(val) => Err(AppError(Some(val.to_string()))),
        };
    }
    Err(AppError(Some(format!(
        "take_from_mysql_row {} not found",
        name
    ))))
}

pub fn take_datetime_from_mysql_row(row: &mut Row, name: &str) -> Result<String, AppError> {
    if let Some(val) = row.take_opt::<NaiveDateTime, &str>(name) {
        return match val {
            Ok(val) => Ok(val.format(DATE_TIME_FORMAT).to_string()),
            Err(val) => Err(AppError(Some(val.to_string()))),
        };
    }
    Err(AppError(Some(format!(
        "take_datetime_from_mysql_row {} not found",
        name
    ))))
}

pub fn take_some_datetime_from_mysql_row(
    row: &mut Row,
    name: &str,
) -> Result<Option<String>, AppError> {
    if let Some(val) = row.take_opt::<Option<NaiveDateTime>, &str>(name) {
        return match val {
            Ok(val) => {
                if let Some(val) = val {
                    Ok(Some(val.format(DATE_TIME_FORMAT).to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(val) => Err(AppError(Some(val.to_string()))),
        };
    }
    Err(AppError(Some(format!(
        "take_some_datetime_from_mysql_row {} not found",
        name
    ))))
}

pub fn take_json_from_mysql_row<T: DeserializeOwned>(
    row: &mut Row,
    name: &str,
) -> Result<T, AppError> {
    if let Some(val) = row.take_opt::<String, &str>(name) {
        if let Ok(val) = val {
            if val.len() == 0 {
                return Err(AppError(None));
            }
            let val: serde_json::Result<T> = serde_json::from_str(&val);
            return match val {
                Err(e) => Err(AppError(Some(e.to_string()))),
                Ok(v) => Ok(v),
            };
        }

        return Err(AppError(None));
    }

    Err(AppError(None))
}

pub fn option_take_json_from_mysql_row<T: DeserializeOwned>(
    row: &mut Row,
    name: &str,
) -> Option<T> {
    if let Ok(v) = take_json_from_mysql_row(row, name) {
        Some(v)
    } else {
        None
    }
}

pub fn to_json_string_for_mysql<T: Serialize>(val: &T) -> Result<String, AppError> {
    let val: serde_json::Result<String> = serde_json::to_string(&val);
    match val {
        Err(e) => Err(AppError(Some(e.to_string()))),
        Ok(v) => Ok(v),
    }
}

pub fn option_to_json_string_for_mysql<T: Serialize>(val: &Option<T>) -> Option<String> {
    if let Some(val) = val {
        if let Ok(v) = to_json_string_for_mysql(val) {
            Some(v)
        } else {
            None
        }
    } else {
        None
    }
}
