use crate::app::repositories::{UserPaginateParams, UserRepository, UserRepositoryError};
use crate::{HashService, MysqlPool, NewUserData, PaginationResult, RandomService, User};
use actix_web::web::Data;
use actix_web::{error, Error};
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub struct UserService {
    hash_service: Data<HashService>,
    user_repository: Data<UserRepository>,
}

impl UserService {
    pub fn new(hash_service: Data<HashService>, user_repository: Data<UserRepository>) -> Self {
        Self {
            hash_service,
            user_repository,
        }
    }

    pub fn first_by_id(&self, user_id: u64) -> Result<Option<User>, UserRepositoryError> {
        self.user_repository.get_ref().first_by_id(user_id)
    }

    pub fn first_by_id_throw_http(&self, user_id: u64) -> Result<User, Error> {
        let user = self
            .first_by_id(user_id)
            .map_err(|e| error::ErrorInternalServerError(""))?;
        if let Some(user) = user {
            return Ok(user);
        }
        Err(error::ErrorNotFound(""))
    }

    pub fn first_by_email(&self, email: &str) -> Result<Option<User>, UserRepositoryError> {
        self.user_repository.get_ref().first_by_email(email)
    }

    pub fn create(&self, mut new_user: NewUserData) -> Result<(), UserServiceError> {
        let hash_service = self.hash_service.get_ref();
        let user_repository = self.user_repository.get_ref();

        if let Some(password) = &new_user.password {
            new_user.password = Some(hash_service.hash_password(password).map_err(|e| {
                log::error!("UserService::insert - {e}");
                UserServiceError::PasswordHashFail
            })?);
        }

        let users = vec![new_user];
        user_repository.insert(&users).map_err(|e| match e {
            UserRepositoryError::DuplicateEmail => UserServiceError::DuplicateEmail,
            _ => UserServiceError::Fail,
        })
    }

    pub fn paginate(
        &self,
        params: &UserPaginateParams,
    ) -> Result<PaginationResult<User>, UserRepositoryError> {
        self.user_repository.get_ref().paginate(params)
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
