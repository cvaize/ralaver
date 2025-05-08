use crate::{HashService, MysqlPool, NewUser, User};
use actix_web::web::Data;
use diesel::result::DatabaseErrorKind;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use strum_macros::{Display, EnumString};

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
                log::error!("UserService::first_by_id - {e}");
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
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum UserServiceError {
    DbConnectionFail,
    DuplicateEmail,
    PasswordHashFail,
    Fail,
}
