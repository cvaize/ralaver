use std::str::FromStr;
use actix_web::cookie::time::{format_description, PrimitiveDateTime};
use crate::{take_from_mysql_row, take_some_datetime_from_mysql_row, AppError, Disk, File, FileColumn, FromMysqlDto, MysqlColumnEnum, MysqlIdColumn, MysqlPool, MysqlQueryBuilder, MysqlRepository, PaginateParams, ToMysqlDto};
use actix_web::web::Data;
use chrono::{DateTime, Utc};
use mysql::Row;
use mysql::Value;
use strum_macros::{Display, EnumIter, EnumString};
use crate::helpers::DATE_TIME_FORMAT;

pub struct FileMysqlRepository {
    db_pool: Data<MysqlPool>,
}

impl MysqlRepository<File, FilePaginateParams, FileColumn, FileFilter, FileSort>
    for FileMysqlRepository
{
    fn get_repository_name(&self) -> &str {
        "FileMysqlRepository"
    }
    fn get_table(&self) -> &str {
        "files"
    }
    fn get_db_pool(&self) -> &MysqlPool {
        self.db_pool.get_ref()
    }
}

impl FileMysqlRepository {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        Self { db_pool }
    }

    pub fn first_by_disk_and_local_path(
        &self,
        disk: &Disk,
        local_path: &str,
    ) -> Result<Option<File>, AppError> {
        let filters: Vec<FileFilter> = vec![
            FileFilter::Disk(disk.to_string()),
            FileFilter::LocalPath(local_path.to_string()),
        ];
        self.first_by_filters(&filters)
    }

    pub fn exists_by_disk_and_local_path(&self, disk: &Disk, local_path: &str) -> Result<bool, AppError> {
        let filters: Vec<FileFilter> = vec![
            FileFilter::Disk(disk.to_string()),
            FileFilter::LocalPath(local_path.to_string()),
        ];
        self.exists_by_filters(&filters)
    }

    pub fn delete_by_disk_and_local_path(&self, disk: &Disk, local_path: &str) -> Result<(), AppError> {
        let filters: Vec<FileFilter> = vec![
            FileFilter::Disk(disk.to_string()),
            FileFilter::LocalPath(local_path.to_string()),
        ];
        self.delete_by_filters(&filters)
    }
}

pub type FilePaginateParams = PaginateParams<FileFilter, FileSort>;

#[derive(Debug)]
pub enum FileFilter {
    Id(u64),
    Disk(String),
    LocalPath(String),
    Search(String),
}

impl MysqlQueryBuilder for FileFilter {
    fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::Id(_) => query.push_str("id=:id"),
            Self::Disk(_) => query.push_str("disk=:disk"),
            Self::LocalPath(_) => query.push_str("local_path=:local_path"),
            Self::Search(_) => query.push_str("(name LIKE :search OR local_path LIKE :search)"),
        }
    }

    fn push_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        match self {
            Self::Id(value) => {
                params.push(("id".to_string(), Value::from(value)));
            }
            Self::Disk(value) => {
                params.push(("disk".to_string(), Value::from(value)));
            }
            Self::LocalPath(value) => {
                params.push(("local_path".to_string(), Value::from(value)));
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

impl MysqlQueryBuilder for FileSort {
    fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::IdAsc => query.push_str("id ASC"),
            Self::IdDesc => query.push_str("id DESC"),
        };
    }

    fn push_params_to_vec(&self, _: &mut Vec<(String, Value)>) {}
}

impl ToMysqlDto<FileColumn> for File {
    fn push_mysql_param_to_vec(&self, column: &FileColumn, params: &mut Vec<(String, Value)>) {
        match column {
            FileColumn::Id => params.push((column.to_string(), Value::from(self.id.to_owned()))),
            FileColumn::Filename => {
                params.push((column.to_string(), Value::from(self.filename.to_owned())))
            }
            FileColumn::PublicPath => {
                params.push((column.to_string(), Value::from(self.public_path.to_owned())))
            }
            FileColumn::LocalPath => {
                params.push((column.to_string(), Value::from(self.local_path.to_owned())))
            }
            FileColumn::Mime => {
                params.push((column.to_string(), Value::from(self.mime.to_owned())))
            }
            FileColumn::Hash => {
                params.push((column.to_string(), Value::from(self.hash.to_owned())))
            }
            FileColumn::Size => {
                params.push((column.to_string(), Value::from(self.size.to_owned())))
            }
            FileColumn::CreatorUserId => params.push((
                column.to_string(),
                Value::from(self.creator_user_id.to_owned()),
            )),
            FileColumn::CreatedAt => {
                params.push((column.to_string(), Value::from(self.created_at.to_owned())))
            }
            FileColumn::UpdatedAt => {
                params.push((column.to_string(), Value::from(self.updated_at.to_owned())))
            }
            FileColumn::FileDeleteAt => params.push((
                column.to_string(),
                Value::from(self.file_delete_at.to_owned()),
            )),
            FileColumn::FileDeletedAt => params.push((
                column.to_string(),
                Value::from(self.file_deleted_at.to_owned()),
            )),
            FileColumn::DeletedAt => {
                params.push((column.to_string(), Value::from(self.deleted_at.to_owned())))
            }
            FileColumn::IsDeleted => {
                params.push((column.to_string(), Value::from(self.is_deleted.to_owned())))
            }
            FileColumn::IsPublic => {
                params.push((column.to_string(), Value::from(self.is_public.to_owned())))
            }
            FileColumn::Disk => {
                params.push((column.to_string(), Value::from(self.disk.to_owned())))
            }
        }
    }
    fn get_id(&self) -> u64 {
        self.id
    }
}

impl FromMysqlDto for File {
    fn take_from_mysql_row(row: &mut Row) -> Result<Self, AppError> {
        Ok(Self {
            id: take_from_mysql_row(row, FileColumn::Id.to_string().as_str())?,
            filename: take_from_mysql_row(row, FileColumn::Filename.to_string().as_str())?,
            public_path: take_from_mysql_row(row, FileColumn::PublicPath.to_string().as_str())?,
            local_path: take_from_mysql_row(row, FileColumn::LocalPath.to_string().as_str())?,
            mime: take_from_mysql_row(row, FileColumn::Mime.to_string().as_str())?,
            hash: take_from_mysql_row(row, FileColumn::Hash.to_string().as_str())?,
            size: take_from_mysql_row(row, FileColumn::Size.to_string().as_str())?,
            creator_user_id: take_from_mysql_row(
                row,
                FileColumn::CreatorUserId.to_string().as_str(),
            )?,
            created_at: take_some_datetime_from_mysql_row(row, FileColumn::CreatedAt.to_string().as_str())?,
            updated_at: take_some_datetime_from_mysql_row(row, FileColumn::UpdatedAt.to_string().as_str())?,
            file_delete_at: take_from_mysql_row(
                row,
                FileColumn::FileDeleteAt.to_string().as_str(),
            )?,
            file_deleted_at: take_from_mysql_row(
                row,
                FileColumn::FileDeletedAt.to_string().as_str(),
            )?,
            deleted_at: take_from_mysql_row(row, FileColumn::DeletedAt.to_string().as_str())?,
            is_public: take_from_mysql_row(row, FileColumn::IsPublic.to_string().as_str())?,
            is_deleted: take_from_mysql_row(row, FileColumn::IsDeleted.to_string().as_str())?,
            disk: take_from_mysql_row(row, FileColumn::Disk.to_string().as_str())?,
        })
    }
}

impl MysqlColumnEnum for FileColumn {}
impl MysqlIdColumn for FileColumn {
    fn get_mysql_id_column() -> Self {
        Self::Id
    }
}
