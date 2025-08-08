use crate::helpers::{join_vec, now_date_time_str};
use crate::{
    take_from_mysql_row, take_some_datetime_from_mysql_row, AppError, File, FileColumn, FileFilter,
    FromMysqlDto, MysqlColumnEnum, MysqlIdColumn, MysqlPool, MysqlQueryBuilder, MysqlRepository,
    PaginateParams, ToMysqlDto, UserFile, UserFileColumn,
};
use actix_web::web::Data;
use mysql::Row;
use mysql::Value;
use strum_macros::{Display, EnumIter, EnumString};

pub struct UserFileMysqlRepository {
    db_pool: Data<MysqlPool>,
}

impl MysqlRepository<UserFile, UserFilePaginateParams, UserFileColumn, UserFileFilter, UserFileSort>
    for UserFileMysqlRepository
{
    fn get_repository_name(&self) -> &str {
        "UserFileMysqlRepository"
    }
    fn get_table(&self) -> &str {
        "users_files"
    }
    fn get_db_pool(&self) -> &MysqlPool {
        self.db_pool.get_ref()
    }
}

impl UserFileMysqlRepository {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        Self { db_pool }
    }

    pub fn first_by_user_id_and_file_id(
        &self,
        user_id: u64,
        file_id: u64,
    ) -> Result<Option<UserFile>, AppError> {
        let filters: Vec<UserFileFilter> = vec![
            UserFileFilter::UserId(user_id),
            UserFileFilter::FileId(file_id),
        ];
        self.first(&filters)
    }

    pub fn exists_by_user_id_and_file_id(
        &self,
        user_id: u64,
        file_id: u64,
    ) -> Result<bool, AppError> {
        let filters: Vec<UserFileFilter> = vec![
            UserFileFilter::UserId(user_id),
            UserFileFilter::FileId(file_id),
        ];
        self.exists(&filters)
    }

    pub fn soft_delete_by_id(&self, id: u64) -> Result<(), AppError> {
        let filters = vec![UserFileFilter::Id(id), UserFileFilter::IsDeleted(false)];

        let mut data = UserFile::default();
        data.deleted_at = Some(now_date_time_str());
        data.is_deleted = true;

        let columns: Option<Vec<UserFileColumn>> =
            Some(vec![UserFileColumn::DeletedAt, UserFileColumn::IsDeleted]);

        self.update(&filters, &data, &columns)
    }

    pub fn soft_delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), AppError> {
        let filters = vec![
            UserFileFilter::Ids(ids.clone()),
            UserFileFilter::IsDeleted(false),
        ];

        let mut data = UserFile::default();
        data.deleted_at = Some(now_date_time_str());
        data.is_deleted = true;

        let columns: Option<Vec<UserFileColumn>> =
            Some(vec![UserFileColumn::DeletedAt, UserFileColumn::IsDeleted]);

        self.update(&filters, &data, &columns)
    }

    pub fn restore_by_id(&self, id: u64) -> Result<(), AppError> {
        let filters = vec![UserFileFilter::Id(id), UserFileFilter::IsDeleted(true)];

        let mut data = UserFile::default();
        data.deleted_at = None;
        data.is_deleted = false;

        let columns: Option<Vec<UserFileColumn>> =
            Some(vec![UserFileColumn::DeletedAt, UserFileColumn::IsDeleted]);

        self.update(&filters, &data, &columns)
    }

    pub fn restore_by_ids(&self, ids: &Vec<u64>) -> Result<(), AppError> {
        let filters = vec![
            UserFileFilter::Ids(ids.clone()),
            UserFileFilter::IsDeleted(true),
        ];

        let mut data = UserFile::default();
        data.deleted_at = None;
        data.is_deleted = false;

        let columns: Option<Vec<UserFileColumn>> =
            Some(vec![UserFileColumn::DeletedAt, UserFileColumn::IsDeleted]);

        self.update(&filters, &data, &columns)
    }

    pub fn soft_delete_by_file_id(&self, file_id: u64) -> Result<(), AppError> {
        let filters = vec![
            UserFileFilter::FileId(file_id),
            UserFileFilter::IsDeleted(false),
        ];

        let mut data = UserFile::default();
        data.deleted_at = Some(now_date_time_str());
        data.is_deleted = true;

        let columns: Option<Vec<UserFileColumn>> =
            Some(vec![UserFileColumn::DeletedAt, UserFileColumn::IsDeleted]);

        self.update(&filters, &data, &columns)
    }

    pub fn soft_delete_by_file_ids(&self, file_ids: &Vec<u64>) -> Result<(), AppError> {
        let filters = vec![
            UserFileFilter::FileIds(file_ids.clone()),
            UserFileFilter::IsDeleted(false),
        ];

        let mut data = UserFile::default();
        data.deleted_at = Some(now_date_time_str());
        data.is_deleted = true;

        let columns: Option<Vec<UserFileColumn>> =
            Some(vec![UserFileColumn::DeletedAt, UserFileColumn::IsDeleted]);

        self.update(&filters, &data, &columns)
    }
}

pub type UserFilePaginateParams = PaginateParams<UserFileFilter, UserFileSort>;

#[derive(Debug, Clone)]
pub enum UserFileFilter {
    Id(u64),
    Ids(Vec<u64>),
    UserId(u64),
    FileId(u64),
    FileIds(Vec<u64>),
    Path(String),
    Filename(String),
    Search(String),
    IsDeleted(bool),
    IsPublic(bool),
}

impl MysqlQueryBuilder for UserFileFilter {
    fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::Id(_) => query.push_str("id=:f_id"),
            Self::Ids(value) => {
                let v = format!("id in ({})", join_vec(value, ","));
                query.push_str(&v)
            },
            Self::UserId(_) => query.push_str("user_id=:f_user_id"),
            Self::FileId(_) => query.push_str("file_id=:f_file_id"),
            Self::FileIds(value) => {
                let v = format!("file_id in ({})", join_vec(value, ","));
                query.push_str(&v)
            },
            Self::Path(_) => query.push_str("path=:f_path"),
            Self::Filename(_) => query.push_str("filename=:f_filename"),
            Self::Search(_) => query.push_str("(filename LIKE :f_search OR upload_filename LIKE :f_search OR path LIKE :f_search)"),
            Self::IsDeleted(_) => query.push_str("is_deleted=:f_is_deleted"),
            Self::IsPublic(_) => query.push_str("is_public=:f_is_public"),
        }
    }

    fn push_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        match self {
            Self::Id(value) => {
                params.push(("f_id".to_string(), Value::from(value)));
            }
            Self::Ids(_) => {}
            Self::UserId(value) => {
                params.push(("f_user_id".to_string(), Value::from(value)));
            }
            Self::FileId(value) => {
                params.push(("f_file_id".to_string(), Value::from(value)));
            }
            Self::FileIds(_) => {}
            Self::Path(value) => {
                params.push(("f_path".to_string(), Value::from(value)));
            }
            Self::Filename(value) => {
                params.push(("f_filename".to_string(), Value::from(value)));
            }
            Self::Search(value) => {
                let mut s = "%".to_string();
                s.push_str(value);
                s.push_str("%");
                params.push(("f_search".to_string(), Value::from(s)));
            }
            Self::IsDeleted(value) => {
                params.push(("f_is_deleted".to_string(), Value::from(value)));
            }
            Self::IsPublic(value) => {
                params.push(("f_is_public".to_string(), Value::from(value)));
            }
        }
    }
}

#[derive(Debug, Display, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum UserFileSort {
    IdAsc,
    IdDesc,
}

impl MysqlQueryBuilder for UserFileSort {
    fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::IdAsc => query.push_str("id ASC"),
            Self::IdDesc => query.push_str("id DESC"),
        };
    }

    fn push_params_to_vec(&self, _: &mut Vec<(String, Value)>) {}
}

impl ToMysqlDto<UserFileColumn> for UserFile {
    fn push_mysql_param_to_vec(&self, column: &UserFileColumn, params: &mut Vec<(String, Value)>) {
        match column {
            UserFileColumn::Id => {
                params.push((column.to_string(), Value::from(self.id.to_owned())))
            }
            UserFileColumn::UserId => {
                params.push((column.to_string(), Value::from(self.user_id.to_owned())))
            }
            UserFileColumn::FileId => {
                params.push((column.to_string(), Value::from(self.file_id.to_owned())))
            }
            UserFileColumn::Filename => {
                params.push((column.to_string(), Value::from(self.filename.to_owned())))
            }
            UserFileColumn::Path => {
                params.push((column.to_string(), Value::from(self.path.to_owned())))
            }
            UserFileColumn::UploadFilename => params.push((
                column.to_string(),
                Value::from(self.upload_filename.to_owned()),
            )),
            UserFileColumn::Mime => {
                params.push((column.to_string(), Value::from(self.mime.to_owned())))
            }
            UserFileColumn::CreatedAt => {
                params.push((column.to_string(), Value::from(self.created_at.to_owned())))
            }
            UserFileColumn::UpdatedAt => {
                params.push((column.to_string(), Value::from(self.updated_at.to_owned())))
            }
            UserFileColumn::DeletedAt => {
                params.push((column.to_string(), Value::from(self.deleted_at.to_owned())))
            }
            UserFileColumn::IsDeleted => {
                params.push((column.to_string(), Value::from(self.is_deleted.to_owned())))
            }
            UserFileColumn::IsPublic => {
                params.push((column.to_string(), Value::from(self.is_public.to_owned())))
            }
            UserFileColumn::Disk => {
                params.push((column.to_string(), Value::from(self.disk.to_owned())))
            }
        }
    }
    fn get_id(&self) -> u64 {
        self.id
    }
}

impl FromMysqlDto for UserFile {
    fn take_from_mysql_row(row: &mut Row) -> Result<Self, AppError> {
        Ok(Self {
            id: take_from_mysql_row(row, UserFileColumn::Id.to_string().as_str())?,
            user_id: take_from_mysql_row(row, UserFileColumn::UserId.to_string().as_str())?,
            file_id: take_from_mysql_row(row, UserFileColumn::FileId.to_string().as_str())?,
            filename: take_from_mysql_row(row, UserFileColumn::Filename.to_string().as_str())?,
            path: take_from_mysql_row(row, UserFileColumn::Path.to_string().as_str())?,
            upload_filename: take_from_mysql_row(
                row,
                UserFileColumn::UploadFilename.to_string().as_str(),
            )?,
            mime: take_from_mysql_row(row, UserFileColumn::Mime.to_string().as_str())?,
            created_at: take_some_datetime_from_mysql_row(
                row,
                UserFileColumn::CreatedAt.to_string().as_str(),
            )?,
            updated_at: take_some_datetime_from_mysql_row(
                row,
                UserFileColumn::UpdatedAt.to_string().as_str(),
            )?,
            deleted_at: take_some_datetime_from_mysql_row(
                row,
                UserFileColumn::DeletedAt.to_string().as_str(),
            )?,
            is_deleted: take_from_mysql_row(row, UserFileColumn::IsDeleted.to_string().as_str())?,
            is_public: take_from_mysql_row(row, UserFileColumn::IsPublic.to_string().as_str())?,
            disk: take_from_mysql_row(row, UserFileColumn::Disk.to_string().as_str())?,
        })
    }
}

impl MysqlColumnEnum for UserFileColumn {}
impl MysqlIdColumn for UserFileColumn {
    fn get_mysql_id_column() -> Self {
        Self::Id
    }
}
