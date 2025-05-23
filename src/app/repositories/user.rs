use crate::app::repositories::make_pagination_mysql_query;
use crate::{
    AuthServiceError, MysqlPool, NewUser, PaginationResult, RandomService, UpdateUser, User,
    UserServiceError,
};
use actix_web::web::Data;
use r2d2_mysql::mysql::prelude::Queryable;
use r2d2_mysql::mysql::Value;
use r2d2_mysql::mysql::{params, Error, Params, Row};
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

static COLUMNS_QUERY: &str = "id, email, locale, surname, name, patronymic";

static SELECT_BY_ID_QUERY: &str =
    "SELECT id, email, locale, surname, name, patronymic FROM users WHERE id=:id";

static SELECT_BY_EMAIL_QUERY: &str =
    "SELECT id, email, locale, surname, name, patronymic FROM users WHERE email=:email";

static SELECT_CREDENTIALS_BY_EMAIL_QUERY: &str =
    "SELECT id, email, password FROM users WHERE email=:email";

static EXISTS_BY_EMAIL_QUERY: &str =
    "SELECT EXISTS(SELECT 1 FROM users WHERE email=:email LIMIT 1) as is_exists";

static PAGINATION_QUERY: &str =
    "SELECT id, email, locale, surname, name, patronymic, COUNT(*) OVER () as total_records FROM users LIMIT :per_page OFFSET :offset";

static INSERT_QUERY: &str =
    "INSERT INTO users (email, password, locale, surname, name, patronymic) VALUES (:email, :password, :locale, :surname, :name, :patronymic)";

static DELETE_BY_ID_QUERY: &str = "DELETE FROM users WHERE id=:id";

static UPDATE_PASSWORD_BY_EMAIL_QUERY: &str =
    "UPDATE users SET password=:password  WHERE email=:email";

static UPDATE_BY_EMAIL_QUERY: [&str; 2] = ["UPDATE users SET ", " WHERE email=:email"];

pub struct UserRepository {
    db_pool: Data<MysqlPool>,
}

impl UserRepository {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        Self { db_pool }
    }

    pub fn first_by_id(&self, id: u64) -> Result<Option<User>, UserRepositoryError> {
        let mut conn = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserRepository::first_by_id - {e}");
            return UserRepositoryError::DbConnectionFail;
        })?;

        let row: Option<Row> = conn
            .exec_first(
                SELECT_BY_ID_QUERY,
                params! {
                    "id" => id,
                },
            )
            .map_err(|_| UserRepositoryError::Fail)?;

        if let Some(row) = row {
            return Ok(Some(User::from_db_row(&row)));
        }

        Ok(None)
    }

    pub fn first_by_email(&self, email: &str) -> Result<Option<User>, UserRepositoryError> {
        let mut conn = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserRepository::first_by_email - {e}");
            return UserRepositoryError::DbConnectionFail;
        })?;

        let row: Option<Row> = conn
            .exec_first(
                SELECT_BY_EMAIL_QUERY,
                params! {
                    "email" => email,
                },
            )
            .map_err(|_| UserRepositoryError::Fail)?;

        if let Some(row) = row {
            return Ok(Some(User::from_db_row(&row)));
        }

        Ok(None)
    }

    pub fn credentials_by_email(&self, email: &str) -> Result<Option<User>, UserRepositoryError> {
        let mut conn = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserRepository::credentials_by_email - {e}");
            return UserRepositoryError::DbConnectionFail;
        })?;

        let row: Option<Row> = conn
            .exec_first(
                SELECT_CREDENTIALS_BY_EMAIL_QUERY,
                params! {
                    "email" => email,
                },
            )
            .map_err(|_| UserRepositoryError::Fail)?;

        if let Some(row) = row {
            return Ok(Some(User::from_db_row(&row)));
        }

        Ok(None)
    }

    pub fn paginate(
        &self,
        params: &UserPaginateParams,
    ) -> Result<PaginationResult<User>, UserRepositoryError> {
        let mut conn = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserRepository::paginate - {e}");
            return UserRepositoryError::DbConnectionFail;
        })?;
        let page = params.page;
        let per_page = params.per_page;
        let offset = (page - 1) * per_page;

        let mut mysql_where: String = String::new();
        let mut mysql_params: Vec<(String, Value)> = vec![
            (String::from("per_page"), Value::Int(per_page)),
            (String::from("offset"), Value::Int(offset)),
        ];

        if let Some(filter) = params.filter {
            filter.push_params_to_vec(&mut mysql_params);
            filter.push_params_to_mysql_query(&mut mysql_where);
        }

        let sql = make_pagination_mysql_query(COLUMNS_QUERY, "users", &mysql_where);

        // dbg!(&sql);
        // dbg!(&mysql_params);
        let rows = conn
            .exec_iter(&sql, Params::from(mysql_params))
            .map_err(|e| {
                log::error!("UserRepository::paginate - {e}");
                UserRepositoryError::Fail
            })?;

        let mut records: Vec<User> = Vec::new();
        let mut total_records: i64 = 0;
        for row in rows.into_iter() {
            if let Ok(row) = row {
                records.push(User::from_db_row(&row));
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
        let mut conn = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserRepository::exists_by_email - {e}");
            return UserRepositoryError::DbConnectionFail;
        })?;

        let row: Option<Row> = conn
            .exec_first(
                EXISTS_BY_EMAIL_QUERY,
                params! {
                    "email" => email,
                },
            )
            .map_err(|_| UserRepositoryError::Fail)?;

        if let Some(row) = row {
            return Ok(row.get("is_exists").unwrap_or(false));
        }

        Ok(false)
    }

    pub fn insert(&self, users: &Vec<NewUser>) -> Result<(), UserRepositoryError> {
        let mut conn = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserRepository::insert - {e}");
            return UserRepositoryError::DbConnectionFail;
        })?;

        conn.exec_batch(
            INSERT_QUERY,
            users.iter().map(|u| {
                params! {
                    "email" => &u.email,
                    "password" => &u.password,
                    "locale" => &u.locale,
                    "surname" => &u.surname,
                    "name" => &u.name,
                    "patronymic" => &u.patronymic,
                }
            }),
        )
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
        let mut conn = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserRepository::delete_by_id - {e}");
            return UserRepositoryError::DbConnectionFail;
        })?;

        conn.exec_drop(DELETE_BY_ID_QUERY, params! { "id" => id, })
            .map_err(|e| {
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
        let mut conn = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserRepository::update_password_by_email - {e}");
            return UserRepositoryError::DbConnectionFail;
        })?;

        conn.exec_drop(
            UPDATE_PASSWORD_BY_EMAIL_QUERY,
            params! { "email" => email, "password" => password },
        )
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
pub struct UserFilter<'a> {
    pub search: &'a Option<String>,
}

impl UserFilter<'_> {
    pub fn push_params_to_mysql_query(&self, query: &mut String) {
        if self.search.is_some() {
            query.push_str("(email LIKE :search OR surname LIKE :search OR name LIKE :search OR patronymic LIKE :search)");
        }
    }

    pub fn push_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        if let Some(search) = self.search {
            let mut s = "%".to_string();
            s.push_str(search.trim());
            s.push_str("%");
            params.push((
                String::from("search"),
                Value::Bytes(s.into_bytes()),
            ));
        }
    }
}

#[derive(Debug)]
pub enum UserSort {
    IdAsc,
    IdDesc,
}

#[derive(Debug)]
pub struct UserPaginateParams<'a> {
    pub page: i64,
    pub per_page: i64,
    pub filter: Option<&'a UserFilter<'a>>,
    pub sort: Option<&'a UserSort>,
}

impl<'a> UserPaginateParams<'a> {
    pub fn new(
        page: i64,
        per_page: i64,
        filter: Option<&'a UserFilter>,
        sort: Option<&'a UserSort>,
    ) -> Self {
        Self {
            page,
            per_page,
            filter,
            sort,
        }
    }

    pub fn simple(page: i64, per_page: i64) -> Self {
        Self {
            page,
            per_page,
            filter: None,
            sort: None,
        }
    }

    pub fn one() -> Self {
        Self {
            page: 1,
            per_page: 1,
            filter: None,
            sort: None,
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

        let users = vec![NewUser::empty(email.to_string())];
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

        let mut users: Vec<NewUser> = Vec::new();

        for email in emails {
            users.push(NewUser::empty(email.to_string()));
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

            let users = vec![NewUser::empty(email.to_string())];
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

        let users = vec![NewUser::empty(email.to_string())];
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

        let users = vec![NewUser::empty(email.to_string())];
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
        let users = vec![NewUser::empty(email.to_string())];
        user_rep.insert(&users).unwrap();

        // Search exists
        let search = Some("paginate_with_filters".to_string());
        let filter = UserFilter { search: &search };
        let mut params = UserPaginateParams::one();
        params.filter = Some(&filter);

        let users = user_rep.paginate(&params).unwrap();
        assert_eq!(users.records[0].email, email);

        // Search not exists
        let search = Some("paginate_____filters".to_string());
        let filter = UserFilter { search: &search };
        let mut params = UserPaginateParams::one();
        params.filter = Some(&filter);

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
