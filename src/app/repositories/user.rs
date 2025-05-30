use crate::app::repositories::{
    make_delete_mysql_query, make_insert_mysql_query, make_is_exists_mysql_query,
    make_pagination_mysql_query, make_select_mysql_query, make_update_mysql_query, FromDbRowError,
    FromMysqlDto, ToMysqlDto,
};
use crate::{
    MysqlPool, MysqlPooledConnection, PaginationResult, User,
};
use actix_web::web::Data;
use r2d2_mysql::mysql::prelude::Queryable;
use r2d2_mysql::mysql::Value;
use r2d2_mysql::mysql::{params, Error, Params, Row};
use strum_macros::{Display, EnumIter, EnumString};

pub struct UserRepository {
    table: String,
    db_pool: Data<MysqlPool>,
}

impl UserRepository {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        let table = "users".to_string();
        Self { table, db_pool }
    }

    fn connection(&self) -> Result<MysqlPooledConnection, UserRepositoryError> {
        self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserRepository::connection - {e}");
            return UserRepositoryError::DbConnectionFail;
        })
    }

    fn row_to_entity(&self, row: &mut Row) -> Result<User, UserRepositoryError> {
        User::take_from_db_row(row).map_err(|_| UserRepositoryError::Fail)
    }

    fn try_row_to_entity(
        &self,
        row: &mut Option<Row>,
    ) -> Result<Option<User>, UserRepositoryError> {
        if let Some(row) = row {
            return Ok(Some(self.row_to_entity(row)?));
        }

        Ok(None)
    }

    fn try_row_is_exists(&self, row: &Option<Row>) -> Result<bool, UserRepositoryError> {
        if let Some(row) = row {
            return Ok(row.get("is_exists").unwrap_or(false));
        }

        Ok(false)
    }

    pub fn first_by_id(&self, id: u64) -> Result<Option<User>, UserRepositoryError> {
        let columns = User::db_select_columns();
        let query = make_select_mysql_query(&self.table, &columns, "id=:id", "");
        let mut conn = self.connection()?;
        let mut row: Option<Row> = conn
            .exec_first(query, params! {"id" => id})
            .map_err(|_| UserRepositoryError::Fail)?;

        self.try_row_to_entity(&mut row)
    }

    pub fn first_by_email(&self, email: &str) -> Result<Option<User>, UserRepositoryError> {
        let columns = User::db_select_columns();
        let query = make_select_mysql_query(&self.table, &columns, "email=:email", "");
        let mut conn = self.connection()?;
        let mut row: Option<Row> = conn
            .exec_first(query, params! {"email" => email})
            .map_err(|_| UserRepositoryError::Fail)?;

        self.try_row_to_entity(&mut row)
    }

    pub fn paginate(
        &self,
        params: &UserPaginateParams,
    ) -> Result<PaginationResult<User>, UserRepositoryError> {
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
        let columns = User::db_select_columns();
        let query = make_pagination_mysql_query(table, &columns, &mysql_where, &mysql_order);

        let rows = conn
            .exec_iter(&query, Params::from(mysql_params))
            .map_err(|e| {
                log::error!("UserRepository::paginate - {e}");
                UserRepositoryError::Fail
            })?;

        let mut records: Vec<User> = Vec::new();
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

    pub fn exists_by_email(&self, email: &str) -> Result<bool, UserRepositoryError> {
        let mut conn = self.connection()?;
        let table = &self.table;
        let query = make_is_exists_mysql_query(&table, "email=:email");
        let row: Option<Row> = conn
            .exec_first(query, params! { "email" => email })
            .map_err(|_| UserRepositoryError::Fail)?;

        self.try_row_is_exists(&row)
    }

    pub fn insert(&self, data: &User) -> Result<(), UserRepositoryError> {
        let mut conn = self.connection()?;
        let columns = User::db_insert_columns();
        let params = data.to_insert_db_params();
        let query = make_insert_mysql_query(&self.table, &columns);
        conn.exec_drop(query, params)
            .map_err(|e| match &e {
                Error::MySqlError(e_) => {
                    if e_.code == 1062 {
                        UserRepositoryError::DuplicateEmail
                    } else {
                        log::error!("UserRepository::insert - {e}");
                        UserRepositoryError::Fail
                    }
                }
                _ => {
                    log::error!("UserRepository::insert - {e}");
                    UserRepositoryError::Fail
                }
            })?;

        Ok(())
    }

    pub fn delete_by_id(&self, id: u64) -> Result<(), UserRepositoryError> {
        let mut conn = self.connection()?;
        let query = make_delete_mysql_query(&self.table, "id=:id");
        conn.exec_drop(query, params! { "id" => id }).map_err(|e| {
            log::error!("UserRepository::delete_by_id - {e}");
            return UserRepositoryError::Fail;
        })?;

        Ok(())
    }

    pub fn delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), UserRepositoryError> {
        let mut conn = self.connection()?;
        let query = make_delete_mysql_query(&self.table, "id IN (:id)");
        let params = ids.iter().map(|id| params! { "id" => id });
        conn.exec_batch(query, params).map_err(|e| {
            log::error!("UserRepository::delete_by_ids - {e}");
            return UserRepositoryError::Fail;
        })?;

        Ok(())
    }

    pub fn delete_by_email(&self, email: &str) -> Result<(), UserRepositoryError> {
        let mut conn = self.connection()?;
        let query = make_delete_mysql_query(&self.table, "email=:email");
        conn.exec_drop(query, params! { "email" => email }).map_err(|e| {
            log::error!("UserRepository::delete_by_email - {e}");
            return UserRepositoryError::Fail;
        })?;

        Ok(())
    }

    pub fn update<'a>(&self, data: &User) -> Result<(), UserRepositoryError> {
        let mut conn = self.connection()?;
        let columns = User::db_update_columns();
        let params = data.to_update_db_params();

        let query = make_update_mysql_query(&self.table, &columns, "id=:id");
        conn.exec_drop(query, params).map_err(|e| {
            log::error!("UserRepository::update - {e}");
            return UserRepositoryError::Fail;
        })?;

        Ok(())
    }

    pub fn update_password_by_email(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(), UserRepositoryError> {
        let mut conn = self.connection()?;
        let query = make_update_mysql_query(&self.table, "password=:password", "email=:email");
        conn.exec_drop(query, params! { "email" => email, "password" => password })
            .map_err(|e| {
                log::error!("UserRepository::update_password_by_email - {e}");
                return UserRepositoryError::Fail;
            })?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum UserRepositoryError {
    DbConnectionFail,
    DuplicateEmail,
    NotFound,
    Fail,
}

#[derive(Debug)]
pub enum UserFilter<'a> {
    Id(u64),
    Email(&'a str),
    Search(&'a str),
    Locale(&'a str),
}

impl UserFilter<'_> {
    pub fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::Id(_) => query.push_str("id=:id"),
            Self::Email(_) => query.push_str("email=:email"),
            Self::Search(_) => query.push_str("(email LIKE :search OR surname LIKE :search OR name LIKE :search OR patronymic LIKE :search)"),
            Self::Locale(_) => query.push_str("locale=:locale"),
        }
    }

    pub fn push_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        match self {
            Self::Id(value) => {
                params.push(("id".to_string(), Value::UInt(value.to_owned())));
            }
            Self::Email(value) => {
                params.push((
                    "email".to_string(),
                    Value::Bytes(value.to_string().into_bytes()),
                ));
            }
            Self::Search(value) => {
                let mut s = "%".to_string();
                s.push_str(value);
                s.push_str("%");
                params.push(("search".to_string(), Value::Bytes(s.into_bytes())));
            }
            Self::Locale(value) => {
                params.push((
                    "locale".to_string(),
                    Value::Bytes(value.to_string().into_bytes()),
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

impl UserSort {
    pub fn push_params_to_mysql_query(&self, query: &mut String) {
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

    pub fn push_params_to_vec(&self, _: &mut Vec<(String, Value)>) {}
}

#[derive(Debug)]
pub struct UserPaginateParams<'a> {
    pub page: i64,
    pub per_page: i64,
    pub filters: Vec<UserFilter<'a>>,
    pub sort: Option<UserSort>,
}

impl<'a> UserPaginateParams<'a> {
    pub fn new(
        page: i64,
        per_page: i64,
        filters: Vec<UserFilter<'a>>,
        sort: Option<UserSort>,
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

impl ToMysqlDto for User {
    fn db_select_columns() -> String {
        "id,email,password,locale,surname,name,patronymic".to_string()
    }
    fn db_insert_columns() -> String {
        "(email, password, locale, surname, name, patronymic) VALUES (:email, :password, :locale, :surname, :name, :patronymic)".to_string()
    }
    fn to_insert_db_params(&self) -> Params {
        params! {
            "email" => &self.email,
            "password" => &self.password,
            "locale" => &self.locale,
            "surname" => &self.surname,
            "name" => &self.name,
            "patronymic" => &self.patronymic,
        }
    }
    fn db_update_columns() -> String {
        "email=:email, password=:password, locale=:locale, surname=:surname, name=:name, patronymic=:patronymic".to_string()
    }
    fn to_update_db_params(&self) -> Params {
        params! {
            "id" => &self.id,
            "email" => &self.email,
            "password" => &self.password,
            "locale" => &self.locale,
            "surname" => &self.surname,
            "name" => &self.name,
            "patronymic" => &self.patronymic,
        }
    }
}

impl FromMysqlDto for User {
    fn take_from_db_row(row: &mut Row) -> Result<Self, FromDbRowError> {
        Ok(Self {
            id: row.take("id").ok_or(FromDbRowError)?,
            password: row.take("password").ok_or(FromDbRowError)?,
            email: row.take("email").ok_or(FromDbRowError)?,
            locale: row.take("locale").unwrap_or(None),
            surname: row.take("surname").unwrap_or(None),
            name: row.take("name").unwrap_or(None),
            patronymic: row.take("patronymic").unwrap_or(None),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preparation;

    #[test]
    fn test_first_by_id() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_repository.get_ref();

        let email = "admin_first_by_id@admin.example";

        user_rep.delete_by_email(email).unwrap();

        let user = User::empty(email.to_string());
        user_rep.insert(&user).unwrap();

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
        let user_rep = all_services.user_repository.get_ref();

        let email = "admin_first_by_email@admin.example";

        user_rep.delete_by_email(email).unwrap();

        let user = User::empty(email.to_string());
        user_rep.insert(&user).unwrap();

        let user = user_rep.first_by_email(email).unwrap().unwrap();
        assert_eq!(user.email, email);

        user_rep.delete_by_email(email).unwrap();
    }

    #[test]
    fn test_paginate() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_repository.get_ref();

        let users = user_rep.paginate(&UserPaginateParams::one()).unwrap();
        assert_eq!(users.page, 1);
    }

    #[test]
    fn test_insert() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_repository.get_ref();
        let emails = ["admin_insert1@admin.example", "admin_insert2@admin.example"];

        let mut users: Vec<User> = Vec::new();

        for email in emails {
            users.push(User::empty(email.to_string()));
            user_rep.delete_by_email(email).unwrap();
        }

        for user in users {
            user_rep.insert(&user).unwrap();
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
        let user_rep = all_services.user_repository.get_ref();
        let emails = [
            "admin_delete_by_id1@admin.example",
            "admin_delete_by_id2@admin.example",
        ];

        for email in emails {
            user_rep.delete_by_email(email).unwrap();

            let user = User::empty(email.to_string());
            user_rep.insert(&user).unwrap();

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
                if let Some(user) = user {
                    is_not_exists = false;
                }
            }
            assert!(is_not_exists);
        }
    }

    #[test]
    fn test_delete_by_ids() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_repository.get_ref();
        let emails = [
            "admin_delete_by_ids1@admin.example",
            "admin_delete_by_ids2@admin.example",
        ];

        let mut ids: Vec<u64> = Vec::new();
        for email in emails {
            user_rep.delete_by_email(email).unwrap();

            let user = User::empty(email.to_string());
            user_rep.insert(&user).unwrap();

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
                if let Some(user) = user {
                    is_not_exists = false;
                }
            }
            assert!(is_not_exists);
        }
    }

    #[test]
    fn test_update_password_by_email() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_repository.get_ref();

        let email = "admin_update_password_by_email@admin.example";

        user_rep.delete_by_email(email).unwrap();

        let user = User::empty(email.to_string());
        user_rep.insert(&user).unwrap();

        let user = user_rep.first_by_email(email);
        let mut is_exists = false;
        if let Ok(user) = user {
            if let Some(user) = user {
                is_exists = true;
                assert!(user.password.is_none());
            }
        }
        assert!(is_exists);

        user_rep.update_password_by_email(email, email).unwrap();

        let user = user_rep.first_by_email(email);
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
        let user_rep = all_services.user_repository.get_ref();

        let email = "admin_exists_by_email@admin.example";

        user_rep.delete_by_email(email).unwrap();

        assert_eq!(user_rep.exists_by_email(email).unwrap(), false);

        let user = User::empty(email.to_string());
        user_rep.insert(&user).unwrap();

        assert!(user_rep.exists_by_email(email).unwrap());

        user_rep.delete_by_email(email).unwrap();
    }

    #[test]
    fn test_paginate_with_filters() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_repository.get_ref();

        let email = "admin_paginate_with_filters@admin.example";

        user_rep.delete_by_email(email).unwrap();

        // Create temp data
        let user = User::empty(email.to_string());
        user_rep.insert(&user).unwrap();

        // Search exists
        let search = "paginate_with_filters";
        let mut params = UserPaginateParams::one();
        params.filters.push(UserFilter::Search(search));

        let users = user_rep.paginate(&params).unwrap();
        assert_eq!(users.records[0].email, email);

        // Search not exists
        let search = "paginate_____filters";
        let mut params = UserPaginateParams::one();
        params.filters.push(UserFilter::Search(search));

        let users = user_rep.paginate(&params).unwrap();
        assert_eq!(users.records.len(), 0);

        // Delete temp data
        user_rep.delete_by_email(email).unwrap();
    }

    #[test]
    fn test_update() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_repository.get_ref();

        let emails = [
            "admin_update1@admin.example",
            "admin_update2@admin.example",
        ];

        for email in emails {
            user_rep.delete_by_email(email).unwrap();
        }

        let user_data = User::empty(emails[0].to_string());
        user_rep.insert(&user_data).unwrap();

        let mut user = user_rep.first_by_email(emails[0]).unwrap().unwrap();
        assert_eq!(emails[0].to_string(), user.email);

        user.email = emails[1].to_string();

        user_rep.update(&user).unwrap();

        let user = user_rep.first_by_id(user.id).unwrap().unwrap();
        assert_eq!(emails[1].to_string(), user.email);

        for email in emails {
            user_rep.delete_by_email(email).unwrap();
        }
    }
}
