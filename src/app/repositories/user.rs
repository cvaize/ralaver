use crate::{
    make_select_mysql_query, make_update_mysql_query, option_take_json_from_mysql_row,
    option_to_json_string_for_mysql, take_from_mysql_row, AppError, FromMysqlDto,
    MysqlAllColumnEnum, MysqlColumnEnum, MysqlIdColumn, MysqlPool, MysqlQueryBuilder,
    MysqlRepository, PaginateParams, ToMysqlDto, User, UserColumn, UserCredentials,
    UserCredentialsColumn,
};
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

    fn try_row_to_credentials(
        &self,
        row: &mut Option<Row>,
    ) -> Result<Option<UserCredentials>, AppError> {
        if let Some(row) = row {
            return Ok(Some(self.row_to_credentials(row)?));
        }

        Ok(None)
    }

    pub fn first_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let filters: Vec<UserFilter> = vec![UserFilter::Email(email.to_string())];
        self.first_by_filters(&filters)
    }

    pub fn exists_by_email(&self, email: &str) -> Result<bool, AppError> {
        let filters: Vec<UserFilter> = vec![UserFilter::Email(email.to_string())];
        self.exists_by_filters(&filters)
    }

    pub fn delete_by_email(&self, email: &str) -> Result<(), AppError> {
        let filters: Vec<UserFilter> = vec![UserFilter::Email(email.to_string())];
        self.delete_by_filters(&filters)
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
}

pub type UserPaginateParams = PaginateParams<UserFilter, UserSort>;

#[derive(Debug)]
pub enum UserFilter {
    Id(u64),
    Email(String),
    Search(String),
    Locale(String),
}

impl MysqlQueryBuilder for UserFilter {
    fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::Id(_) => query.push_str("id=:id"),
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
            locale: take_from_mysql_row(row, UserColumn::Locale.to_string().as_str())
                .unwrap_or(None),
            surname: take_from_mysql_row(row, UserColumn::Surname.to_string().as_str())
                .unwrap_or(None),
            name: take_from_mysql_row(row, UserColumn::Name.to_string().as_str()).unwrap_or(None),
            patronymic: take_from_mysql_row(row, UserColumn::Patronymic.to_string().as_str())
                .unwrap_or(None),
            is_super_admin: take_from_mysql_row(row, UserColumn::IsSuperAdmin.to_string().as_str())
                .unwrap_or(false),
            roles_ids: option_take_json_from_mysql_row(
                row,
                UserColumn::RolesIds.to_string().as_str(),
            ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preparation;

    #[test]
    fn test_first_by_id() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::user::tests::test_first_by_id
        let (_, all_services) = preparation();
        let user_rep = all_services.user_mysql_repository.get_ref();

        let email = "admin_first_by_id@admin.example";

        user_rep.delete_by_email(email).unwrap();

        let user = User::empty(email.to_string());
        user_rep.insert_one(&user).unwrap();

        let user = user_rep.first_by_email(email).unwrap().unwrap();
        assert_eq!(user.email, email);
        let id = user.id;

        let user = user_rep.first_by_id(id).unwrap().unwrap();
        assert_eq!(user.id, id);

        user_rep.delete_by_email(email).unwrap();
    }

    #[test]
    fn test_first_by_email() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_mysql_repository.get_ref();

        let email = "admin_first_by_email@admin.example";

        user_rep.delete_by_email(email).unwrap();

        let user = User::empty(email.to_string());
        user_rep.insert_one(&user).unwrap();

        let user = user_rep.first_by_email(email).unwrap().unwrap();
        assert_eq!(user.email, email);

        user_rep.delete_by_email(email).unwrap();
    }

    #[test]
    fn test_paginate() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_mysql_repository.get_ref();

        let users = user_rep.paginate(&UserPaginateParams::one()).unwrap();
        assert_eq!(users.page, 1);
    }

    #[test]
    fn test_insert() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::user::tests::test_insert
        let (_, all_services) = preparation();
        let user_rep = all_services.user_mysql_repository.get_ref();
        let emails = ["admin_insert1@admin.example", "admin_insert2@admin.example"];

        let mut users: Vec<User> = Vec::new();

        for email in emails {
            users.push(User::empty(email.to_string()));
            user_rep.delete_by_email(email).unwrap();
        }

        for user in users {
            user_rep.insert_one(&user).unwrap();
        }

        for email in emails {
            let user = user_rep.first_by_email(email);
            let mut is_exists = false;
            if let Ok(user) = user {
                if let Some(user) = user {
                    is_exists = true;
                    user_rep.delete_by_id(user.id).unwrap();
                }
            }
            assert!(is_exists);
        }
    }

    #[test]
    fn test_delete_by_id() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_mysql_repository.get_ref();
        let emails = [
            "admin_delete_by_id1@admin.example",
            "admin_delete_by_id2@admin.example",
        ];

        for email in emails {
            user_rep.delete_by_email(email).unwrap();

            let user = User::empty(email.to_string());
            user_rep.insert_one(&user).unwrap();

            let user = user_rep.first_by_email(email);
            let mut is_exists = false;
            if let Ok(user) = user {
                if let Some(user) = user {
                    is_exists = true;
                    user_rep.delete_by_id(user.id).unwrap();
                }
            }
            assert!(is_exists);

            let user = user_rep.first_by_email(email);
            let mut is_not_exists = true;
            if let Ok(user) = user {
                if let Some(_) = user {
                    is_not_exists = false;
                }
            }
            assert!(is_not_exists);
        }
    }

    #[test]
    fn test_delete_by_ids() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_mysql_repository.get_ref();
        let emails = [
            "admin_delete_by_ids1@admin.example",
            "admin_delete_by_ids2@admin.example",
        ];

        let mut ids: Vec<u64> = Vec::new();
        for email in emails {
            user_rep.delete_by_email(email).unwrap();

            let user = User::empty(email.to_string());
            user_rep.insert_one(&user).unwrap();

            let user = user_rep.first_by_email(email);
            let mut is_exists = false;
            if let Ok(user) = user {
                if let Some(user) = user {
                    is_exists = true;
                    ids.push(user.id);
                }
            }
            assert!(is_exists);
        }
        user_rep.delete_by_ids(&ids).unwrap();

        for email in emails {
            let user = user_rep.first_by_email(email);
            let mut is_not_exists = true;
            if let Ok(user) = user {
                if let Some(_) = user {
                    is_not_exists = false;
                }
            }
            assert!(is_not_exists);
        }
    }

    #[test]
    fn test_update_password_by_email() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_mysql_repository.get_ref();

        let email = "admin_update_password_by_email@admin.example";

        user_rep.delete_by_email(email).unwrap();

        let user = User::empty(email.to_string());
        user_rep.insert_one(&user).unwrap();

        let user = user_rep.first_credentials_by_email(email);
        let mut is_exists = false;
        if let Ok(user) = user {
            if let Some(user) = user {
                is_exists = true;
                assert!(user.password.is_none());
            }
        }
        assert!(is_exists);

        user_rep.update_password_by_email(email, email).unwrap();

        let user = user_rep.first_credentials_by_email(email);
        let mut is_exists = false;
        if let Ok(user) = user {
            if let Some(user) = user {
                is_exists = true;
                assert_eq!(user.password.unwrap(), email);
            }
        }
        assert!(is_exists);

        user_rep.delete_by_email(email).unwrap();
    }

    #[test]
    fn test_exists_by_email() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_mysql_repository.get_ref();

        let email = "admin_exists_by_email@admin.example";

        user_rep.delete_by_email(email).unwrap();

        assert_eq!(user_rep.exists_by_email(email).unwrap(), false);

        let user = User::empty(email.to_string());
        user_rep.insert_one(&user).unwrap();

        assert!(user_rep.exists_by_email(email).unwrap());

        user_rep.delete_by_email(email).unwrap();
    }

    #[test]
    fn test_paginate_with_filters() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_mysql_repository.get_ref();

        let email = "admin_paginate_with_filters@admin.example";

        user_rep.delete_by_email(email).unwrap();

        // Create temp data
        let user = User::empty(email.to_string());
        user_rep.insert_one(&user).unwrap();

        // Search exists
        let search = "paginate_with_filters";
        let mut params = UserPaginateParams::one();
        params.filters.push(UserFilter::Search(search.to_string()));

        let users = user_rep.paginate(&params).unwrap();
        assert_eq!(users.records[0].email, email);

        // Search not exists
        let search = "paginate_____filters";
        let mut params = UserPaginateParams::one();
        params.filters.push(UserFilter::Search(search.to_string()));

        let users = user_rep.paginate(&params).unwrap();
        assert_eq!(users.records.len(), 0);

        // Delete temp data
        user_rep.delete_by_email(email).unwrap();
    }

    #[test]
    fn test_update() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::user::tests::test_update
        let (_, all_services) = preparation();
        let user_rep = all_services.user_mysql_repository.get_ref();

        let emails = ["admin_update1@admin.example", "admin_update2@admin.example"];

        for email in emails {
            user_rep.delete_by_email(email).unwrap();
        }

        let user_data = User::empty(emails[0].to_string());
        user_rep.insert_one(&user_data).unwrap();

        let mut user = user_rep.first_by_email(emails[0]).unwrap().unwrap();
        assert_eq!(emails[0].to_string(), user.email);

        user.email = emails[1].to_string();

        let columns: Option<Vec<UserColumn>> = None;
        user_rep.update_one(&user, &columns).unwrap();

        let user = user_rep.first_by_id(user.id).unwrap().unwrap();
        assert_eq!(emails[1].to_string(), user.email);

        for email in emails {
            user_rep.delete_by_email(email).unwrap();
        }
    }
}
