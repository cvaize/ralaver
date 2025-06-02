use crate::{
    MysqlPool, MysqlPooledConnection, PaginationResult, Role,
    make_delete_mysql_query, make_insert_mysql_query, make_is_exists_mysql_query,
    make_pagination_mysql_query, make_select_mysql_query, make_update_mysql_query, FromDbRowError,
    FromMysqlDto, ToMysqlDto,
};
use actix_web::web::Data;
use r2d2_mysql::mysql::prelude::Queryable;
use r2d2_mysql::mysql::Value;
use r2d2_mysql::mysql::{params, Error, Params, Row};
use strum_macros::{Display, EnumIter, EnumString};

pub struct RoleMysqlRepository {
    table: String,
    db_pool: Data<MysqlPool>,
}

impl RoleMysqlRepository {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        let table = "roles".to_string();
        Self { table, db_pool }
    }

    fn connection(&self) -> Result<MysqlPooledConnection, RoleMysqlRepositoryError> {
        self.db_pool.get_ref().get().map_err(|e| {
            log::error!("RoleRepository::connection - {e}");
            return RoleMysqlRepositoryError::DbConnectionFail;
        })
    }

    fn row_to_entity(&self, row: &mut Row) -> Result<Role, RoleMysqlRepositoryError> {
        Role::take_from_mysql_row(row).map_err(|_| RoleMysqlRepositoryError::Fail)
    }

    fn try_row_to_entity(
        &self,
        row: &mut Option<Row>,
    ) -> Result<Option<Role>, RoleMysqlRepositoryError> {
        if let Some(row) = row {
            return Ok(Some(self.row_to_entity(row)?));
        }

        Ok(None)
    }

    fn try_row_is_exists(&self, row: &Option<Row>) -> Result<bool, RoleMysqlRepositoryError> {
        if let Some(row) = row {
            return Ok(row.get("is_exists").unwrap_or(false));
        }

        Ok(false)
    }

    pub fn first_by_id(&self, id: u64) -> Result<Option<Role>, RoleMysqlRepositoryError> {
        let columns = Role::mysql_select_columns();
        let query = make_select_mysql_query(&self.table, &columns, "id=:id", "");
        let mut conn = self.connection()?;
        let mut row: Option<Row> = conn
            .exec_first(query, params! {"id" => id})
            .map_err(|_| RoleMysqlRepositoryError::Fail)?;

        self.try_row_to_entity(&mut row)
    }

    pub fn first_by_code(&self, code: &str) -> Result<Option<Role>, RoleMysqlRepositoryError> {
        let columns = Role::mysql_select_columns();
        let query = make_select_mysql_query(&self.table, &columns, "code=:code", "");
        let mut conn = self.connection()?;
        let mut row: Option<Row> = conn
            .exec_first(query, params! {"code" => code})
            .map_err(|_| RoleMysqlRepositoryError::Fail)?;

        self.try_row_to_entity(&mut row)
    }

    pub fn paginate(
        &self,
        params: &RoleMysqlRepositoryPaginateParams,
    ) -> Result<PaginationResult<Role>, RoleMysqlRepositoryError> {
        let mut conn = self.connection()?;
        let page = params.page;
        let per_page = params.per_page;
        let offset = (page - 1) * per_page;

        let mut mysql_where: String = String::new();
        let mut mysql_order: String = String::new();
        let mut mysql_params: Vec<(String, Value)> = vec![
            (String::from("per_page"), Value::Int(per_page)),
            (String::from("offset"), Value::Int(offset)),
        ];

        let mut is_and = false;
        for filter in &params.filters {
            if is_and {
                mysql_where.push_str(" AND ")
            }
            filter.push_params_to_vec(&mut mysql_params);
            filter.push_params_to_mysql_query(&mut mysql_where);
            is_and = true;
        }

        if let Some(sort) = &params.sort {
            sort.push_params_to_vec(&mut mysql_params);
            sort.push_params_to_mysql_query(&mut mysql_order);
        }

        let table = &self.table;
        let columns = Role::mysql_select_columns();
        let query = make_pagination_mysql_query(table, &columns, &mysql_where, &mysql_order);

        let rows = conn
            .exec_iter(&query, Params::from(mysql_params))
            .map_err(|e| {
                log::error!("RoleRepository::paginate - {e}");
                RoleMysqlRepositoryError::Fail
            })?;

        let mut records: Vec<Role> = Vec::new();
        let mut total_records: i64 = 0;
        for mut row in rows.into_iter() {
            if let Ok(row) = &mut row {
                if total_records == 0 {
                    total_records = row.take("total_records").unwrap_or(total_records);
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

    pub fn exists_by_code(&self, code: &str) -> Result<bool, RoleMysqlRepositoryError> {
        let mut conn = self.connection()?;
        let table = &self.table;
        let query = make_is_exists_mysql_query(&table, "code=:code");
        let row: Option<Row> = conn
            .exec_first(query, params! { "code" => code })
            .map_err(|_| RoleMysqlRepositoryError::Fail)?;

        self.try_row_is_exists(&row)
    }

    pub fn insert(&self, data: &Role) -> Result<(), RoleMysqlRepositoryError> {
        let mut conn = self.connection()?;
        let columns = Role::mysql_insert_columns();
        let params = data.to_insert_mysql_params();
        let query = make_insert_mysql_query(&self.table, &columns);
        conn.exec_drop(query, params)
            .map_err(|e| match &e {
                Error::MySqlError(e_) => {
                    if e_.code == 1062 {
                        RoleMysqlRepositoryError::DuplicateCode
                    } else {
                        log::error!("RoleRepository::insert - {e}");
                        RoleMysqlRepositoryError::Fail
                    }
                }
                _ => {
                    log::error!("RoleRepository::insert - {e}");
                    RoleMysqlRepositoryError::Fail
                }
            })?;

        Ok(())
    }

    pub fn delete_by_id(&self, id: u64) -> Result<(), RoleMysqlRepositoryError> {
        let mut conn = self.connection()?;
        let query = make_delete_mysql_query(&self.table, "id=:id");
        conn.exec_drop(query, params! { "id" => id }).map_err(|e| {
            log::error!("RoleRepository::delete_by_id - {e}");
            return RoleMysqlRepositoryError::Fail;
        })?;

        Ok(())
    }

    pub fn delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), RoleMysqlRepositoryError> {
        let mut conn = self.connection()?;
        let query = make_delete_mysql_query(&self.table, "id IN (:id)");
        let params = ids.iter().map(|id| params! { "id" => id });
        conn.exec_batch(query, params).map_err(|e| {
            log::error!("RoleRepository::delete_by_ids - {e}");
            return RoleMysqlRepositoryError::Fail;
        })?;

        Ok(())
    }

    pub fn delete_by_code(&self, code: &str) -> Result<(), RoleMysqlRepositoryError> {
        let mut conn = self.connection()?;
        let query = make_delete_mysql_query(&self.table, "code=:code");
        conn.exec_drop(query, params! { "code" => code }).map_err(|e| {
            log::error!("RoleRepository::delete_by_code - {e}");
            return RoleMysqlRepositoryError::Fail;
        })?;

        Ok(())
    }

    pub fn update<'a>(&self, data: &Role) -> Result<(), RoleMysqlRepositoryError> {
        let mut conn = self.connection()?;
        let columns = Role::mysql_update_columns();
        let params = data.to_update_mysql_params();
        // TODO: With permissions

        let query = make_update_mysql_query(&self.table, &columns, "id=:id");
        conn.exec_drop(query, params).map_err(|e| {
            log::error!("RoleRepository::update - {e}");
            return RoleMysqlRepositoryError::Fail;
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum RoleMysqlRepositoryError {
    DbConnectionFail,
    DuplicateCode,
    NotFound,
    Fail,
}

#[derive(Debug)]
pub enum RoleMysqlRepositoryFilter<'a> {
    Id(u64),
    Code(&'a str),
    Search(&'a str),
}

impl RoleMysqlRepositoryFilter<'_> {
    pub fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::Id(_) => query.push_str("id=:id"),
            Self::Code(_) => query.push_str("code=:code"),
            Self::Search(_) => query.push_str("(name LIKE :search OR code LIKE :search)"),
        }
    }

    pub fn push_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        match self {
            Self::Id(value) => {
                params.push(("id".to_string(), Value::UInt(value.to_owned())));
            }
            Self::Code(value) => {
                params.push((
                    "code".to_string(),
                    Value::Bytes(value.to_string().into_bytes()),
                ));
            }
            Self::Search(value) => {
                let mut s = "%".to_string();
                s.push_str(value);
                s.push_str("%");
                params.push(("search".to_string(), Value::Bytes(s.into_bytes())));
            }
        }
    }
}

#[derive(Debug, Display, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum RoleMysqlRepositorySort {
    IdAsc,
    IdDesc,
    NameAsc,
    NameDesc,
    CodeAsc,
    CodeDesc,
}

impl RoleMysqlRepositorySort {
    pub fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::IdAsc => query.push_str("id ASC"),
            Self::IdDesc => query.push_str("id DESC"),
            Self::NameAsc => query.push_str("name ASC"),
            Self::NameDesc => query.push_str("name DESC"),
            Self::CodeAsc => query.push_str("code ASC"),
            Self::CodeDesc => query.push_str("code DESC"),
        };
    }

    pub fn push_params_to_vec(&self, _: &mut Vec<(String, Value)>) {}
}

#[derive(Debug)]
pub struct RoleMysqlRepositoryPaginateParams<'a> {
    pub page: i64,
    pub per_page: i64,
    pub filters: Vec<RoleMysqlRepositoryFilter<'a>>,
    pub sort: Option<RoleMysqlRepositorySort>,
}

impl<'a> RoleMysqlRepositoryPaginateParams<'a> {
    pub fn new(
        page: i64,
        per_page: i64,
        filters: Vec<RoleMysqlRepositoryFilter<'a>>,
        sort: Option<RoleMysqlRepositorySort>,
    ) -> Self {
        Self {
            page,
            per_page,
            filters,
            sort,
        }
    }

    pub fn simple(page: i64, per_page: i64) -> Self {
        Self {
            page,
            per_page,
            filters: Vec::new(),
            sort: None,
        }
    }

    pub fn one() -> Self {
        Self {
            page: 1,
            per_page: 1,
            filters: Vec::new(),
            sort: None,
        }
    }
}

impl ToMysqlDto for Role {
    fn mysql_select_columns() -> String {
        "id,name,code,description,permissions".to_string()
    }
    fn mysql_insert_columns() -> String {
        "(name, code, description, permissions) VALUES (:name, :code, :description, :permissions)".to_string()
    }
    fn to_insert_mysql_params(&self) -> Params {
        let permissions_ = serde_json::to_string(&self.permissions).unwrap_or("".to_string());
        let mut permissions: Option<String> = None;
        if permissions_.len() > 0 {
            permissions = Some(permissions_);
        }
        params! {
            "name" => &self.name,
            "code" => &self.code,
            "description" => &self.description,
            "permissions" => permissions,
        }
    }
    fn mysql_update_columns() -> String {
        "name=:name, code=:code, description=:description, permissions=:permissions".to_string()
    }
    fn to_update_mysql_params(&self) -> Params {
        let permissions_ = serde_json::to_string(&self.permissions).unwrap_or("".to_string());
        let mut permissions: Option<String> = None;
        if permissions_.len() > 0 {
            permissions = Some(permissions_);
        }
        params! {
            "id" => &self.id,
            "name" => &self.name,
            "code" => &self.code,
            "description" => &self.description,
            "permissions" => permissions,
        }
    }
}

impl FromMysqlDto for Role {
    fn take_from_mysql_row(row: &mut Row) -> Result<Self, FromDbRowError> {
        let mut permissions: Option<Vec<String>> = None;

        if let Some(permissions_) = row.take::<String, &str>("permissions") {
            let permissions_: serde_json::Result<Vec<String>> = serde_json::from_str(&permissions_);
            if let Ok(permissions_) = permissions_ {
                permissions = Some(permissions_);
            }
        }

        Ok(Self {
            id: row.take("id").ok_or(FromDbRowError)?,
            name: row.take("name").ok_or(FromDbRowError)?,
            code: row.take("code").ok_or(FromDbRowError)?,
            description: row.take("description").unwrap_or(None),
            permissions
        })
    }
}

trait ShortToMysqlDto {
    fn mysql_short_select_columns() -> String {
        "".to_string()
    }
}

impl ShortToMysqlDto for Role {
    fn mysql_short_select_columns() -> String {
        "id,name,code".to_string()
    }
}