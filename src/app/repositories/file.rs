use crate::{make_delete_mysql_query, make_insert_mysql_query, make_is_exists_mysql_query, make_pagination_mysql_query, make_select_mysql_query, make_update_mysql_query, option_take_json_from_mysql_row, option_to_json_string_for_mysql, take_from_mysql_row, take_json_from_mysql_row, FromDbRowError, FromMysqlDto, MysqlAllColumnEnum, MysqlColumnEnum, MysqlPool, MysqlPooledConnection, PaginationResult, File, FileColumn, ToMysqlDto, UserColumn, PaginateParams};
use actix_web::web::Data;
use r2d2_mysql::mysql::prelude::Queryable;
use r2d2_mysql::mysql::Value;
use r2d2_mysql::mysql::{params, Error, Params, Row};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

pub struct FileMysqlRepository {
    table: String,
    db_pool: Data<MysqlPool>,
}

impl FileMysqlRepository {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        let table = "files".to_string();
        Self { table, db_pool }
    }

    fn connection(&self) -> Result<MysqlPooledConnection, FileRepositoryError> {
        self.db_pool.get_ref().get().map_err(|e| {
            log::error!("FileRepository::connection - {e}");
            return FileRepositoryError::DbConnectionFail;
        })
    }

    fn row_to_entity(&self, row: &mut Row) -> Result<File, FileRepositoryError> {
        File::take_from_mysql_row(row).map_err(|_| FileRepositoryError::Fail)
    }

    fn try_row_to_entity(
        &self,
        row: &mut Option<Row>,
    ) -> Result<Option<File>, FileRepositoryError> {
        if let Some(row) = row {
            return Ok(Some(self.row_to_entity(row)?));
        }

        Ok(None)
    }

    fn try_row_is_exists(&self, row: &Option<Row>) -> Result<bool, FileRepositoryError> {
        if let Some(row) = row {
            return Ok(row.get("is_exists").unwrap_or(false));
        }

        Ok(false)
    }

    pub fn first_by_id(&self, id: u64) -> Result<Option<File>, FileRepositoryError> {
        let columns = FileColumn::mysql_all_select_columns();
        let query = make_select_mysql_query(&self.table, &columns, "id=:id", "");
        let mut conn = self.connection()?;
        let mut row: Option<Row> = conn
            .exec_first(query, params! {"id" => id})
            .map_err(|_| FileRepositoryError::Fail)?;

        self.try_row_to_entity(&mut row)
    }

    pub fn first_by_url(&self, url: &str) -> Result<Option<File>, FileRepositoryError> {
        let columns = FileColumn::mysql_all_select_columns();
        let query = make_select_mysql_query(&self.table, &columns, "url=:url", "");
        let mut conn = self.connection()?;
        let mut row: Option<Row> = conn
            .exec_first(query, params! {"url" => url})
            .map_err(|_| FileRepositoryError::Fail)?;

        self.try_row_to_entity(&mut row)
    }

    pub fn paginate(
        &self,
        params: &FilePaginateParams,
    ) -> Result<PaginationResult<File>, FileRepositoryError> {
        let mut conn = self.connection()?;
        let page = params.page;
        let per_page = params.per_page;
        let offset = (page - 1) * per_page;

        let mut mysql_where: String = String::new();
        let mut mysql_order: String = String::new();
        let mut mysql_params: Vec<(String, Value)> = vec![
            (String::from("per_page"), Value::from(per_page)),
            (String::from("offset"), Value::from(offset)),
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
        let columns = FileColumn::mysql_all_select_columns();
        let query = make_pagination_mysql_query(table, &columns, &mysql_where, &mysql_order);

        let rows = conn
            .exec_iter(&query, Params::from(mysql_params))
            .map_err(|e| {
                log::error!("FileRepository::paginate - {e}");
                FileRepositoryError::Fail
            })?;

        let mut records: Vec<File> = Vec::new();
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

    pub fn exists_by_url(&self, url: &str) -> Result<bool, FileRepositoryError> {
        let mut conn = self.connection()?;
        let table = &self.table;
        let query = make_is_exists_mysql_query(&table, "url=:url");
        let row: Option<Row> = conn
            .exec_first(query, params! { "url" => url })
            .map_err(|_| FileRepositoryError::Fail)?;

        self.try_row_is_exists(&row)
    }

    pub fn insert(&self, data: &File) -> Result<(), FileRepositoryError> {
        let mut conn = self.connection()?;

        let (columns_str, params) = if data.id == 0 {
            let columns: Option<Vec<FileColumn>> = Some(
                FileColumn::iter()
                    .filter(|c| c.ne(&FileColumn::Id))
                    .collect(),
            );
            let columns_str = columns.mysql_insert_columns();
            let mut params: Vec<(String, Value)> = Vec::new();
            data.push_mysql_params_to_vec(&columns, &mut params);
            (columns_str, params)
        } else {
            let columns_str = FileColumn::mysql_all_insert_columns();
            let mut params: Vec<(String, Value)> = Vec::new();
            data.push_all_mysql_params_to_vec(&mut params);
            (columns_str, params)
        };

        let query = make_insert_mysql_query(&self.table, &columns_str);
        conn.exec_drop(query, Params::from(params))
            .map_err(|e| match &e {
                Error::MySqlError(e_) => {
                    if e_.code == 1062 {
                        FileRepositoryError::DuplicateUrl
                    } else {
                        log::error!("FileRepository::insert - {e}");
                        FileRepositoryError::Fail
                    }
                }
                _ => {
                    log::error!("FileRepository::insert - {e}");
                    FileRepositoryError::Fail
                }
            })?;

        Ok(())
    }

    pub fn delete_by_id(&self, id: u64) -> Result<(), FileRepositoryError> {
        let mut conn = self.connection()?;
        let query = make_delete_mysql_query(&self.table, "id=:id");
        conn.exec_drop(query, params! { "id" => id }).map_err(|e| {
            log::error!("FileRepository::delete_by_id - {e}");
            return FileRepositoryError::Fail;
        })?;

        Ok(())
    }

    pub fn delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), FileRepositoryError> {
        let mut conn = self.connection()?;
        let query = make_delete_mysql_query(&self.table, "id IN (:id)");
        let params = ids.iter().map(|id| params! { "id" => id });
        conn.exec_batch(query, params).map_err(|e| {
            log::error!("FileRepository::delete_by_ids - {e}");
            return FileRepositoryError::Fail;
        })?;

        Ok(())
    }

    pub fn delete_by_url(&self, url: &str) -> Result<(), FileRepositoryError> {
        let mut conn = self.connection()?;
        let query = make_delete_mysql_query(&self.table, "url=:url");
        conn.exec_drop(query, params! { "url" => url })
            .map_err(|e| {
                log::error!("FileRepository::delete_by_url - {e}");
                return FileRepositoryError::Fail;
            })?;

        Ok(())
    }

    pub fn update<'a>(
        &self,
        data: &File,
        columns: &Option<Vec<FileColumn>>,
    ) -> Result<(), FileRepositoryError> {
        let mut conn = self.connection()?;
        let columns_str = columns.mysql_update_columns();
        let mut params: Vec<(String, Value)> = vec![(String::from("id"), Value::from(data.id))];
        data.push_mysql_params_to_vec(columns, &mut params);

        let query = make_update_mysql_query(&self.table, &columns_str, "id=:id");
        conn.exec_drop(query, Params::from(params)).map_err(|e| {
            log::error!("FileRepository::update - {e}");
            return FileRepositoryError::Fail;
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum FileRepositoryError {
    DbConnectionFail,
    DuplicateUrl,
    NotFound,
    Fail,
}

#[derive(Debug)]
pub enum FileFilter<'a> {
    Id(u64),
    Url(&'a str),
    Search(&'a str),
}

impl FileFilter<'_> {
    pub fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::Id(_) => query.push_str("id=:id"),
            Self::Url(_) => query.push_str("url=:url"),
            Self::Search(_) => query.push_str("(name LIKE :search OR url LIKE :search)"),
        }
    }

    pub fn push_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        match self {
            Self::Id(value) => {
                params.push(("id".to_string(), Value::from(value)));
            }
            Self::Url(value) => {
                params.push(("url".to_string(), Value::from(value.to_string())));
            }
            Self::Search(value) => {
                let mut s = "%".to_string();
                s.push_str(value);
                s.push_str("%");
                params.push(("search".to_string(), Value::from(s)));
            }
        }
    }
}

#[derive(Debug, Display, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum FileSort {
    IdAsc,
    IdDesc,
}

impl FileSort {
    pub fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::IdAsc => query.push_str("id ASC"),
            Self::IdDesc => query.push_str("id DESC"),
        };
    }

    pub fn push_params_to_vec(&self, _: &mut Vec<(String, Value)>) {}
}


pub type FilePaginateParams<'a> = PaginateParams<FileFilter<'a>, FileSort>;

impl<'a> FilePaginateParams<'a> {
    pub fn new(
        page: i64,
        per_page: i64,
        filters: Vec<FileFilter<'a>>,
        sort: Option<FileSort>,
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

impl ToMysqlDto<FileColumn> for File {
    fn push_mysql_param_to_vec(&self, column: &FileColumn, params: &mut Vec<(String, Value)>) {
        match column {
            FileColumn::Id => params.push((column.to_string(), Value::from(self.id.to_owned()))),
            FileColumn::Name => {
                params.push((column.to_string(), Value::from(self.name.to_owned())))
            },
            FileColumn::Url => {
                params.push((column.to_string(), Value::from(self.url.to_owned())))
            },
            FileColumn::IsDeleted => {
                params.push((column.to_string(), Value::from(self.is_deleted.to_owned())))
            }
        }
    }
}

impl FromMysqlDto for File {
    fn take_from_mysql_row(row: &mut Row) -> Result<Self, FromDbRowError> {
        Ok(Self {
            id: take_from_mysql_row(row, FileColumn::Id.to_string().as_str())?,
            name: take_from_mysql_row(row, FileColumn::Name.to_string().as_str())?,
            url: take_from_mysql_row(row, FileColumn::Url.to_string().as_str())?,
            is_deleted: take_from_mysql_row(row, FileColumn::IsDeleted.to_string().as_str())?,
        })
    }
}

impl MysqlColumnEnum for FileColumn {}
