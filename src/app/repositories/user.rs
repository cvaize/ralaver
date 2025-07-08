use crate::{make_select_mysql_query, make_update_mysql_query, option_take_json_from_mysql_row, option_to_json_string_for_mysql, take_from_mysql_row, AppError, FromMysqlDto, MysqlAllColumnEnum, MysqlColumnEnum, MysqlIdColumn, MysqlPool, MysqlQueryBuilder, MysqlRepository, PaginateParams, Role, RoleFilter, ToMysqlDto, User, UserColumn, UserCredentials, UserCredentialsColumn, UserServiceError};
use actix_web::web::Data;
use mysql::prelude::Queryable;
use mysql::Value;
use mysql::{params, Row};
use strum_macros::{Display, EnumIter, EnumString};

pub struct UserMysqlRepository {
    db_pool: Data<MysqlPool>,
}

impl MysqlRepository<User, UserPaginateParams, UserColumn, UserFilter, UserSort>
    for UserMysqlRepository
{
    fn get_repository_name(&self) -> &str {
        "UserMysqlRepository"
    }
    fn get_table(&self) -> &str {
        "users"
    }
    fn get_db_pool(&self) -> &MysqlPool {
        self.db_pool.get_ref()
    }
}

impl UserMysqlRepository {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        Self { db_pool }
    }

    fn row_to_credentials(&self, row: &mut Row) -> Result<UserCredentials, AppError> {
        UserCredentials::take_from_mysql_row(row).map_err(|e| {
            self.log_error("row_to_credentials", e.to_string());
            return e;
        })
    }

    pub fn first_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let filters: Vec<UserFilter> = vec![UserFilter::Email(email.to_string())];
        self.first(&filters)
    }

    pub fn exists_by_email(&self, email: &str) -> Result<bool, AppError> {
        let filters: Vec<UserFilter> = vec![UserFilter::Email(email.to_string())];
        self.exists(&filters)
    }

    pub fn delete_by_email(&self, email: &str) -> Result<(), AppError> {
        let filters: Vec<UserFilter> = vec![UserFilter::Email(email.to_string())];
        self.delete(&filters)
    }

    fn try_row_to_credentials(
        &self,
        row: &mut Option<Row>,
    ) -> Result<Option<UserCredentials>, AppError> {
        if let Some(row) = row {
            return Ok(Some(self.row_to_credentials(row)?));
        }

        Ok(None)
    }

    pub fn first_credentials_by_email(
        &self,
        email: &str,
    ) -> Result<Option<UserCredentials>, AppError> {
        let table = self.get_table();
        let columns = UserCredentialsColumn::mysql_all_select_columns();
        let query = make_select_mysql_query(table, &columns, "email=:email", "");
        let mut conn = self.connection()?;
        let mut row: Option<Row> = conn
            .exec_first(query, params! {"email" => email})
            .map_err(|e| self.log_error("first_credentials_by_email", e.to_string()))?;

        self.try_row_to_credentials(&mut row)
    }

    pub fn update_password_by_id(&self, id: u64, password: &str) -> Result<(), AppError> {
        let table = self.get_table();
        let mut conn = self.connection()?;
        let query = make_update_mysql_query(table, "password=:password", "id=:id");
        conn.exec_drop(query, params! { "id" => id, "password" => password })
            .map_err(|e| self.log_error("update_password_by_id", e.to_string()))?;

        Ok(())
    }

    pub fn update_password_by_email(&self, email: &str, password: &str) -> Result<(), AppError> {
        let table = self.get_table();
        let mut conn = self.connection()?;
        let query = make_update_mysql_query(table, "password=:password", "email=:email");
        conn.exec_drop(query, params! { "email" => email, "password" => password })
            .map_err(|e| self.log_error("update_password_by_email", e.to_string()))?;

        Ok(())
    }

    pub fn delete_by_id(&self, id: u64) -> Result<(), AppError> {
        let filters = vec![UserFilter::Id(id)];
        self.delete(&filters)
    }

    pub fn delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), AppError> {
        let filters = vec![UserFilter::Ids(ids.to_owned())];
        self.delete(&filters)
    }
}

pub type UserPaginateParams = PaginateParams<UserFilter, UserSort>;

#[derive(Debug)]
pub enum UserFilter {
    Id(u64),
    Ids(Vec<u64>),
    Email(String),
    Search(String),
    Locale(String),
}

impl MysqlQueryBuilder for UserFilter {
    fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::Id(_) => query.push_str("id=:id"),
            Self::Ids(value) => {
                let mut v = "id in (".to_string();
                let ids: Vec<String> = value.iter().map(|d| d.to_string()).collect();
                let ids: String = ids.join(",").to_string();
                v.push_str(&ids);
                v.push_str(")");
                query.push_str(&v)
            },
            Self::Email(_) => query.push_str("email=:email"),
            Self::Search(_) => query.push_str("(email LIKE :search OR surname LIKE :search OR name LIKE :search OR patronymic LIKE :search)"),
            Self::Locale(_) => query.push_str("locale=:locale"),
        }
    }

    fn push_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        match self {
            Self::Id(value) => {
                params.push(("id".to_string(), Value::from(value.to_owned())));
            }
            Self::Ids(_) => {}
            Self::Email(value) => {
                params.push((
                    "email".to_string(),
                    Value::from(value.to_string().into_bytes()),
                ));
            }
            Self::Search(value) => {
                let mut s = "%".to_string();
                s.push_str(value);
                s.push_str("%");
                params.push(("search".to_string(), Value::from(s.into_bytes())));
            }
            Self::Locale(value) => {
                params.push((
                    "locale".to_string(),
                    Value::from(value.to_string().into_bytes()),
                ));
            }
        }
    }
}

#[derive(Debug, Display, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum UserSort {
    IdAsc,
    IdDesc,
    EmailAsc,
    EmailDesc,
    SurnameAsc,
    SurnameDesc,
    NameAsc,
    NameDesc,
    PatronymicAsc,
    PatronymicDesc,
    FullNameAsc,
    FullNameDesc,
}

impl MysqlQueryBuilder for UserSort {
    fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::IdAsc => query.push_str("id ASC"),
            Self::IdDesc => query.push_str("id DESC"),
            Self::EmailAsc => query.push_str("email ASC"),
            Self::EmailDesc => query.push_str("email DESC"),
            Self::SurnameAsc => query.push_str("surname ASC"),
            Self::SurnameDesc => query.push_str("surname DESC"),
            Self::NameAsc => query.push_str("name ASC"),
            Self::NameDesc => query.push_str("name DESC"),
            Self::PatronymicAsc => query.push_str("patronymic ASC"),
            Self::PatronymicDesc => query.push_str("patronymic DESC"),
            Self::FullNameAsc => query.push_str("surname ASC, name ASC, patronymic ASC"),
            Self::FullNameDesc => query.push_str("surname DESC, name DESC, patronymic DESC"),
        };
    }

    fn push_params_to_vec(&self, _: &mut Vec<(String, Value)>) {}
}

impl ToMysqlDto<UserColumn> for User {
    fn push_mysql_param_to_vec(&self, column: &UserColumn, params: &mut Vec<(String, Value)>) {
        match column {
            UserColumn::Id => params.push((column.to_string(), Value::from(self.id.to_owned()))),
            UserColumn::Email => {
                params.push((column.to_string(), Value::from(self.email.to_owned())))
            }
            UserColumn::Locale => {
                params.push((column.to_string(), Value::from(self.locale.to_owned())))
            }
            UserColumn::Surname => {
                params.push((column.to_string(), Value::from(self.surname.to_owned())))
            }
            UserColumn::Name => {
                params.push((column.to_string(), Value::from(self.name.to_owned())))
            }
            UserColumn::Patronymic => {
                params.push((column.to_string(), Value::from(self.patronymic.to_owned())))
            }
            UserColumn::IsSuperAdmin => params.push((
                column.to_string(),
                Value::from(self.is_super_admin.to_owned()),
            )),
            UserColumn::RolesIds => {
                let roles_ids: Option<String> = option_to_json_string_for_mysql(&self.roles_ids);
                params.push((column.to_string(), Value::from(roles_ids)))
            }
            UserColumn::AvatarId => {
                params.push((column.to_string(), Value::from(self.avatar_id.to_owned())))
            }
        }
    }
    fn get_id(&self) -> u64 {
        self.id
    }
}

impl FromMysqlDto for User {
    fn take_from_mysql_row(row: &mut Row) -> Result<Self, AppError> {
        Ok(Self {
            id: take_from_mysql_row(row, UserColumn::Id.to_string().as_str())?,
            email: take_from_mysql_row(row, UserColumn::Email.to_string().as_str())?,
            locale: take_from_mysql_row(row, UserColumn::Locale.to_string().as_str())?,
            surname: take_from_mysql_row(row, UserColumn::Surname.to_string().as_str())?,
            name: take_from_mysql_row(row, UserColumn::Name.to_string().as_str())?,
            patronymic: take_from_mysql_row(row, UserColumn::Patronymic.to_string().as_str())?,
            is_super_admin: take_from_mysql_row(row, UserColumn::IsSuperAdmin.to_string().as_str())?,
            roles_ids: option_take_json_from_mysql_row(
                row,
                UserColumn::RolesIds.to_string().as_str(),
            ),
            avatar_id: take_from_mysql_row(row, UserColumn::AvatarId.to_string().as_str())?,
        })
    }
}

impl ToMysqlDto<UserCredentialsColumn> for UserCredentials {
    fn push_mysql_param_to_vec(
        &self,
        column: &UserCredentialsColumn,
        params: &mut Vec<(String, Value)>,
    ) {
        match column {
            UserCredentialsColumn::Id => {
                params.push((column.to_string(), Value::from(self.id.to_owned())))
            }
            UserCredentialsColumn::Email => {
                params.push((column.to_string(), Value::from(self.email.to_owned())))
            }
            UserCredentialsColumn::Password => {
                params.push((column.to_string(), Value::from(self.password.to_owned())));
            }
        }
    }
    fn get_id(&self) -> u64 {
        self.id
    }
}

impl FromMysqlDto for UserCredentials {
    fn take_from_mysql_row(row: &mut Row) -> Result<Self, AppError> {
        Ok(Self {
            id: take_from_mysql_row(row, UserCredentialsColumn::Id.to_string().as_str())?,
            email: take_from_mysql_row(row, UserCredentialsColumn::Email.to_string().as_str())?,
            password: take_from_mysql_row(
                row,
                UserCredentialsColumn::Password.to_string().as_str(),
            )
            .unwrap_or(None),
        })
    }
}

impl MysqlColumnEnum for UserColumn {}
impl MysqlIdColumn for UserColumn {
    fn get_mysql_id_column() -> Self {
        Self::Id
    }
}
impl MysqlColumnEnum for UserCredentialsColumn {}
impl MysqlIdColumn for UserCredentialsColumn {
    fn get_mysql_id_column() -> Self {
        Self::Id
    }
}
