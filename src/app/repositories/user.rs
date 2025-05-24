use crate::app::repositories::{make_delete_mysql_query, make_insert_mysql_query, make_is_exists_mysql_query, make_pagination_mysql_query, make_select_mysql_query, make_update_mysql_query, FromDbRowError, FromMysqlDto, ToMysqlDto};
use crate::{AuthServiceError, MysqlPool, MysqlPooledConnection, NewUserData, PaginationResult, CredentialsUserData, RandomService, User, UserServiceError};
use actix_web::web::Data;
use r2d2_mysql::mysql::prelude::Queryable;
use r2d2_mysql::mysql::Value;
use r2d2_mysql::mysql::{params, Error, Params, Row};
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumMessage, EnumString, VariantArray, VariantNames};

pub struct UserRepository {
    table: String,
    columns: String,
    credentials_columns: String,
    insert_columns: String,
    db_pool: Data<MysqlPool>,
}

impl UserRepository {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        let table = "users".to_string();
        let columns = "id, email, locale, surname, name, patronymic".to_string();
        let credentials_columns = "id, email, password".to_string();
        let insert_columns = "(email, password, locale, surname, name, patronymic) VALUES (:email, :password, :locale, :surname, :name, :patronymic)".to_string();
        let update_columns = "password=:password".to_string();
        Self {
            table,
            columns,
            credentials_columns,
            insert_columns,
            db_pool,
        }
    }

    fn connection(&self) -> Result<MysqlPooledConnection, UserRepositoryError> {
        self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserRepository::connection - {e}");
            return UserRepositoryError::DbConnectionFail;
        })
    }

    fn row_to_entity(&self, row: &Row) -> Result<User, UserRepositoryError> {
        User::from_db_row(row).map_err(|_| UserRepositoryError::Fail)
    }

    fn try_row_to_entity(&self, row: &Option<Row>) -> Result<Option<User>, UserRepositoryError> {
        if let Some(row) = row {
            return Ok(Some(self.row_to_entity(row)?));
        }

        Ok(None)
    }

    fn row_to_private_entity(&self, row: &Row) -> Result<CredentialsUserData, UserRepositoryError> {
        CredentialsUserData::from_db_row(row).map_err(|_| UserRepositoryError::Fail)
    }

    fn try_row_to_private_entity(
        &self,
        row: &Option<Row>,
    ) -> Result<Option<CredentialsUserData>, UserRepositoryError> {
        if let Some(row) = row {
            return Ok(Some(self.row_to_private_entity(row)?));
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
        let query = make_select_mysql_query(&self.columns, &self.table, "id=:id", "");
        let mut conn = self.connection()?;
        let row: Option<Row> = conn
            .exec_first(query, params! {"id" => id})
            .map_err(|_| UserRepositoryError::Fail)?;

        self.try_row_to_entity(&row)
    }

    pub fn first_by_email(&self, email: &str) -> Result<Option<User>, UserRepositoryError> {
        let query = make_select_mysql_query(&self.columns, &self.table, "email=:email", "");
        let mut conn = self.connection()?;
        let row: Option<Row> = conn
            .exec_first(query, params! {"email" => email})
            .map_err(|_| UserRepositoryError::Fail)?;

        self.try_row_to_entity(&row)
    }

    pub fn credentials_by_email(
        &self,
        email: &str,
    ) -> Result<Option<CredentialsUserData>, UserRepositoryError> {
        let query =
            make_select_mysql_query(&self.credentials_columns, &self.table, "email=:email", "");
        let mut conn = self.connection()?;
        let row: Option<Row> = conn
            .exec_first(query, params! { "email" => email })
            .map_err(|_| UserRepositoryError::Fail)?;

        self.try_row_to_private_entity(&row)
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

        let columns = &self.columns;
        let table = &self.table;
        let query = make_pagination_mysql_query(columns, table, &mysql_where, &mysql_order);

        let rows = conn
            .exec_iter(&query, Params::from(mysql_params))
            .map_err(|e| {
                log::error!("UserRepository::paginate - {e}");
                UserRepositoryError::Fail
            })?;

        let mut records: Vec<User> = Vec::new();
        let mut total_records: i64 = 0;
        for row in rows.into_iter() {
            if let Ok(row) = row {
                records.push(self.row_to_entity(&row)?);
                if total_records == 0 {
                    total_records = row.get("total_records").unwrap_or(total_records);
                }
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

    pub fn insert(&self, users: &Vec<NewUserData>) -> Result<(), UserRepositoryError> {
        let mut conn = self.connection()?;
        let query = make_insert_mysql_query(&self.table, &self.insert_columns);
        conn.exec_batch(query, users.iter().map(|u| u.to_db_params()))
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

    // pub fn update(
    //     &self,
    //     id: u64,
    //     params: &UpdateUser,
    // ) -> Result<(), UserRepositoryError> {
    //     let query_params = {};
    //     let mut query = UPDATE_BY_EMAIL_QUERY[0].to_owned();
    //
    //     let is_not_empty_params = true;
    //
    //     // if let Some(v) = &params.email {
    //     //
    //     // }
    //     // email: Option<Value<String>>,
    //     // password: Option<Value<String>>,
    //     // locale: Option<Value<String>>,
    //     // surname: Option<Value<String>>,
    //     // name: Option<Value<String>>,
    //     // patronymic: Option<Value<String>>,
    //
    //     if is_not_empty_params {
    //         return Ok(());
    //     }
    //     query.push_str(UPDATE_BY_EMAIL_QUERY[1]);
    //
    //     let mut conn = self.db_pool.get_ref().get().map_err(|e| {
    //         log::error!("UserRepository::update_by_email - {e}");
    //         return UserRepositoryError::DbConnectionFail;
    //     })?;
    //
    //     conn.exec_drop(query, query_params)
    //         .map_err(|e| {
    //             log::error!("UserRepository::update_by_email - {e}");
    //             return UserRepositoryError::Fail;
    //         })?;
    //
    //     Ok(())
    // }

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
    Search(&'a str),
    Locale(&'a str),
}

impl UserFilter<'_> {
    pub fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::Search(_) => query.push_str("(email LIKE :search OR surname LIKE :search OR name LIKE :search OR patronymic LIKE :search)"),
            Self::Locale(_) => query.push_str("locale=:locale"),
        }
    }

    pub fn push_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        match self {
            Self::Search(value) => {
                let mut s = "%".to_string();
                s.push_str(value.trim());
                s.push_str("%");
                params.push((String::from("search"), Value::Bytes(s.into_bytes())));
            }
            Self::Locale(value) => {
                params.push((
                    String::from("locale"),
                    Value::Bytes(value.trim().to_string().into_bytes()),
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
    fn to_db_params(&self) -> Params {
        params! {
            "id" => &self.id,
            "email" => &self.email,
            "locale" => &self.locale,
            "surname" => &self.surname,
            "name" => &self.name,
            "patronymic" => &self.patronymic,
        }
    }
}

impl FromMysqlDto for User {
    fn from_db_row(row: &Row) -> Result<Self, FromDbRowError> {
        Ok(Self {
            id: row.get("id").ok_or(FromDbRowError)?,
            email: row.get("email").ok_or(FromDbRowError)?,
            locale: row.get("locale").unwrap_or(None),
            surname: row.get("surname").unwrap_or(None),
            name: row.get("name").unwrap_or(None),
            patronymic: row.get("patronymic").unwrap_or(None),
        })
    }
}

impl ToMysqlDto for CredentialsUserData {
    fn to_db_params(&self) -> Params {
        params! {
            "id" => &self.id,
            "email" => &self.email,
            "password" => &self.password,
        }
    }
}

impl FromMysqlDto for CredentialsUserData {
    fn from_db_row(row: &Row) -> Result<Self, FromDbRowError> {
        Ok(Self {
            id: row.get("id").ok_or(FromDbRowError)?,
            email: row.get("email").ok_or(FromDbRowError)?,
            password: row.get("password").unwrap_or(None),
        })
    }
}

impl ToMysqlDto for NewUserData {

    fn to_db_params(&self) -> Params {
        params! {
            "email" => &self.email,
            "password" => &self.password,
            "locale" => &self.locale,
            "surname" => &self.surname,
            "name" => &self.name,
            "patronymic" => &self.patronymic,
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::preparation;

    #[test]
    fn test_first_by_id() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_rep.get_ref();

        assert_eq!(user_rep.first_by_id(1).unwrap().unwrap().id, 1);
    }

    #[test]
    fn test_first_by_email() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_rep.get_ref();

        let email = "admin_first_by_email@admin.example";

        let user = user_rep.first_by_email(email);
        if let Ok(user) = user {
            if let Some(user) = user {
                user_rep.delete_by_id(user.id).unwrap();
            }
        }

        let users = vec![NewUserData::empty(email.to_string())];
        user_rep.insert(&users).unwrap();

        let user = user_rep.first_by_email(email);
        assert_eq!(user.unwrap().unwrap().email, email);

        let user = user_rep.first_by_email(email);
        if let Ok(user) = user {
            if let Some(user) = user {
                user_rep.delete_by_id(user.id).unwrap();
            }
        }
    }

    #[test]
    fn test_paginate() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_rep.get_ref();

        let users = user_rep.paginate(&UserPaginateParams::one()).unwrap();
        assert_eq!(users.page, 1);
        assert_eq!(users.records[0].id, 1);
    }

    #[test]
    fn test_insert() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_rep.get_ref();
        let emails = ["admin_insert1@admin.example", "admin_insert2@admin.example"];

        let mut users: Vec<NewUserData> = Vec::new();

        for email in emails {
            users.push(NewUserData::empty(email.to_string()));
            let user = user_rep.first_by_email(email);
            if let Ok(user) = user {
                if let Some(user) = user {
                    user_rep.delete_by_id(user.id).unwrap();
                }
            }
        }

        user_rep.insert(&users).unwrap();

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
        let user_rep = all_services.user_rep.get_ref();
        let emails = [
            "admin_delete_by_id1@admin.example",
            "admin_delete_by_id2@admin.example",
        ];

        for email in emails {
            let user = user_rep.first_by_email(email);
            if let Ok(user) = user {
                if let Some(user) = user {
                    user_rep.delete_by_id(user.id).unwrap();
                }
            }

            let users = vec![NewUserData::empty(email.to_string())];
            user_rep.insert(&users).unwrap();

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
    fn test_update_password_by_email() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_rep.get_ref();

        let email = "admin_update_password_by_email@admin.example";

        let user = user_rep.first_by_email(email);
        if let Ok(user) = user {
            if let Some(user) = user {
                user_rep.delete_by_id(user.id).unwrap();
            }
        }

        let users = vec![NewUserData::empty(email.to_string())];
        user_rep.insert(&users).unwrap();

        let user = user_rep.credentials_by_email(email);
        let mut is_exists = false;
        if let Ok(user) = user {
            if let Some(user) = user {
                is_exists = true;
                assert!(user.password.is_none());
            }
        }
        assert!(is_exists);

        user_rep.update_password_by_email(email, email).unwrap();

        let user = user_rep.credentials_by_email(email);
        let mut is_exists = false;
        if let Ok(user) = user {
            if let Some(user) = user {
                is_exists = true;
                assert_eq!(user.password.unwrap(), email);
            }
        }
        assert!(is_exists);

        let user = user_rep.first_by_email(email);
        if let Ok(user) = user {
            if let Some(user) = user {
                user_rep.delete_by_id(user.id).unwrap();
            }
        }
    }

    #[test]
    fn test_exists_by_email() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_rep.get_ref();

        let email = "admin_exists_by_email@admin.example";

        let user = user_rep.first_by_email(email);
        if let Ok(user) = user {
            if let Some(user) = user {
                user_rep.delete_by_id(user.id).unwrap();
            }
        }

        assert_eq!(user_rep.exists_by_email(email).unwrap(), false);

        let users = vec![NewUserData::empty(email.to_string())];
        user_rep.insert(&users).unwrap();

        assert!(user_rep.exists_by_email(email).unwrap());

        let user = user_rep.first_by_email(email);
        if let Ok(user) = user {
            if let Some(user) = user {
                user_rep.delete_by_id(user.id).unwrap();
            }
        }
    }

    #[test]
    fn test_paginate_with_filters() {
        let (_, all_services) = preparation();
        let user_rep = all_services.user_rep.get_ref();

        let email = "admin_paginate_with_filters@admin.example";

        let user = user_rep.first_by_email(email);
        if let Ok(user) = user {
            if let Some(user) = user {
                user_rep.delete_by_id(user.id).unwrap();
            }
        }

        // Create temp data
        let users = vec![NewUserData::empty(email.to_string())];
        user_rep.insert(&users).unwrap();

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
        let user = user_rep.first_by_email(email);
        if let Ok(user) = user {
            if let Some(user) = user {
                user_rep.delete_by_id(user.id).unwrap();
            }
        }
    }
}
