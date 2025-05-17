use actix_web::{error, Error};
use crate::{HashService, MysqlPool, NewUser, User};
use actix_web::web::Data;
use diesel::result::DatabaseErrorKind;
use diesel::{NotFound, QueryDsl, RunQueryDsl, SelectableHelper};
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::query_dsl::methods::LoadQuery;
use strum_macros::{Display, EnumString};
use crate::schema::users::dsl::users as dsl_users;
use crate::schema::users::dsl::email as dsl_email;
use diesel::ExpressionMethods;
use serde_derive::{Deserialize, Serialize};
use crate::mysql_connection::{Paginate, PaginationResult};

pub struct UserService {
    db_pool: Data<MysqlPool>,
    hash_service: Data<HashService>,
}

impl UserService {
    pub fn new(db_pool: Data<MysqlPool>, hash_service: Data<HashService>) -> Self {
        Self {
            db_pool,
            hash_service,
        }
    }

    pub fn first_by_id(&self, user_id: u64) -> Result<User, UserServiceError> {
        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserService::first_by_id - {e}");
            UserServiceError::DbConnectionFail
        })?;

        let user = crate::schema::users::dsl::users
            .find(user_id)
            .select(User::as_select())
            .first(&mut connection)
            .map_err(|e| {
                if e == NotFound {
                    UserServiceError::NotFound
                } else {
                    UserServiceError::Fail
                }
            })?;
        Ok(user)
    }

    pub fn first_by_id_throw_http(&self, user_id: u64) -> Result<User, Error> {
        self.first_by_id(user_id)
            .map_err(|e| {
                match e {
                    UserServiceError::NotFound => error::ErrorNotFound(""),
                    _ => error::ErrorInternalServerError("")
                }
            })
    }

    pub fn first_by_email(&self, email: &str) -> Result<User, UserServiceError> {
        let mut connection = self.db_pool.get_ref().get().map_err(|e| {
            log::error!("UserService::first_by_email - {e}");
            UserServiceError::DbConnectionFail
        })?;

        let user = dsl_users
            .filter(dsl_email.eq(email))
            .select(User::as_select())
            .first(&mut connection)
            .map_err(|_| {
                UserServiceError::Fail
            })?;
        Ok(user)
    }

    pub fn insert(&self, mut new_user: NewUser) -> Result<(), UserServiceError> {
        let db_pool = self.db_pool.get_ref();
        let hash_service = self.hash_service.get_ref();
        if let Some(password) = &new_user.password {
            new_user.password = Some(hash_service.hash_password(password).map_err(|e| {
                log::error!("UserService::insert - {e}");
                UserServiceError::PasswordHashFail
            })?);
        }
        let mut connection = db_pool.get().map_err(|e| {
            log::error!("UserService::insert - {e}");
            return UserServiceError::DbConnectionFail;
        })?;

        diesel::insert_into(crate::schema::users::table)
            .values(&new_user)
            .execute(&mut connection)
            .map_err(|e: diesel::result::Error| match &e {
                diesel::result::Error::DatabaseError(kind, _) => match &kind {
                    DatabaseErrorKind::UniqueViolation => {
                        let email = &new_user.email;
                        log::info!("UserService::insert - {email} - {e}");
                        UserServiceError::DuplicateEmail
                    }
                    _ => {
                        log::error!("UserService::insert - {e}");
                        UserServiceError::Fail
                    }
                },
                _ => {
                    log::error!("UserService::insert - {e}");
                    UserServiceError::Fail
                }
            })?;
        Ok(())
    }

    pub fn paginate(&self, page: i64, per_page: i64) -> Result<PaginationResult<User>, UserServiceError> {
        let db_pool = self.db_pool.get_ref();

        let mut connection = db_pool.get().map_err(|e| {
            log::error!("UserService::paginate - {e}");
            return UserServiceError::DbConnectionFail;
        })?;

        let result = dsl_users
            // .order(posts::published_at.desc())
            // .filter(posts::published_at.is_not_null())
            // .inner_join(users::table)
            // .select((posts::all_columns, (users::id, users::username)))
            .select(User::as_select())
            .paginate(page, per_page)
            .load_and_count_pages(&mut connection)
            .map_err(|_| {
                UserServiceError::Fail
            })?;


        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum UserServiceError {
    DbConnectionFail,
    DuplicateEmail,
    PasswordHashFail,
    NotFound,
    Fail,
}
