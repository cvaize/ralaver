use crate::app::repositories::{UserPaginateParams, UserRepository, UserRepositoryError};
use crate::{HashService, MysqlPool, PaginationResult, RandomService, User};
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

    pub fn first_by_email_throw_http(&self, email: &str) -> Result<User, Error> {
        let user = self
            .first_by_email(email)
            .map_err(|e| error::ErrorInternalServerError(""))?;
        if let Some(user) = user {
            return Ok(user);
        }
        Err(error::ErrorNotFound(""))
    }

    pub fn user_data_hash_password_(
        &self,
        data: &mut User,
        is_need_hash_password: bool,
    ) -> Result<(), UserServiceError> {
        if is_need_hash_password {
            let hash_service = self.hash_service.get_ref();
            if let Some(password) = &data.password {
                data.password = Some(hash_service.hash_password(password).map_err(|e| {
                    log::error!("UserService::user_data_hash_password_ - {e}");
                    UserServiceError::PasswordHashFail
                })?);
            }
        }

        Ok(())
    }

    fn match_error(&self, e: UserRepositoryError) -> UserServiceError {
        match e {
            UserRepositoryError::DuplicateEmail => UserServiceError::DuplicateEmail,
            _ => UserServiceError::Fail,
        }
    }

    pub fn create(
        &self,
        data: &mut User,
        is_need_hash_password: bool,
    ) -> Result<(), UserServiceError> {
        self.user_data_hash_password_(data, is_need_hash_password)?;

        self.user_repository
            .get_ref()
            .insert(data)
            .map_err(|e| self.match_error(e))
    }

    pub fn update(
        &self,
        data: &mut User,
        is_need_hash_password: bool,
    ) -> Result<(), UserServiceError> {
        self.user_data_hash_password_(data, is_need_hash_password)?;

        self.user_repository
            .get_ref()
            .update(data)
            .map_err(|e| self.match_error(e))
    }

    pub fn upsert(
        &self,
        data: &mut User,
        is_need_hash_password: bool,
    ) -> Result<(), UserServiceError> {
        self.user_data_hash_password_(data, is_need_hash_password)?;

        if data.id == 0 {
            self.user_repository
                .get_ref()
                .insert(data)
                .map_err(|e| self.match_error(e))
        } else {
            self.user_repository
                .get_ref()
                .update(data)
                .map_err(|e| self.match_error(e))
        }
    }

    pub fn paginate(
        &self,
        params: &UserPaginateParams,
    ) -> Result<PaginationResult<User>, UserRepositoryError> {
        self.user_repository.get_ref().paginate(params)
    }

    pub fn paginate_throw_http(
        &self,
        params: &UserPaginateParams,
    ) -> Result<PaginationResult<User>, Error> {
        self.paginate(params).map_err(|e| error::ErrorInternalServerError(""))
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString, PartialEq, Eq)]
pub enum UserServiceError {
    DbConnectionFail,
    DuplicateEmail,
    PasswordHashFail,
    NotFound,
    Fail,
}
