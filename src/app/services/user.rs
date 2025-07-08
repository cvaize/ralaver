use crate::helpers::now_date_time_str;
use crate::{
    make_select_mysql_query, make_update_mysql_query, AppError, AuthServiceError, File, FileColumn,
    FileFilter, FileServiceError, HashService, MysqlRepository, PaginationResult,
    TranslatableError, TranslatorService, User, UserColumn, UserCredentials, UserCredentialsColumn,
    UserFileFilter, UserFilter, UserMysqlRepository, UserPaginateParams,
};
use actix_web::web::Data;
use actix_web::{error, Error};
use mysql::{params, Row};
use strum_macros::{Display, EnumString};

pub struct UserService {
    hash_service: Data<HashService>,
    user_repository: Data<UserMysqlRepository>,
}

impl UserService {
    pub fn new(
        hash_service: Data<HashService>,
        user_repository: Data<UserMysqlRepository>,
    ) -> Self {
        Self {
            hash_service,
            user_repository,
        }
    }

    pub fn first_credentials_by_email(
        &self,
        email: &str,
    ) -> Result<Option<UserCredentials>, UserServiceError> {
        self.user_repository
            .get_ref()
            .first_credentials_by_email(email)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_id(&self, id: u64) -> Result<Option<User>, UserServiceError> {
        let filters = vec![UserFilter::Id(id)];
        self.user_repository
            .get_ref()
            .first(&filters)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_id_throw_http(&self, id: u64) -> Result<User, Error> {
        let user = self
            .first_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(user) = user {
            return Ok(user);
        }
        Err(error::ErrorNotFound(""))
    }

    pub fn first_by_email(&self, email: &str) -> Result<Option<User>, UserServiceError> {
        self.user_repository
            .get_ref()
            .first_by_email(email)
            .map_err(|e| self.match_error(e))
    }

    pub fn exists_by_email(&self, email: &str) -> Result<bool, UserServiceError> {
        self.user_repository
            .get_ref()
            .exists_by_email(email)
            .map_err(|e| self.match_error(e))
    }

    pub fn delete_by_email(&self, email: &str) -> Result<(), UserServiceError> {
        self.user_repository
            .get_ref()
            .delete_by_email(email)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_email_throw_http(&self, email: &str) -> Result<User, Error> {
        let user = self
            .first_by_email(email)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(user) = user {
            return Ok(user);
        }
        Err(error::ErrorNotFound(""))
    }

    fn match_error(&self, e: AppError) -> UserServiceError {
        let error = e.to_string();

        if error.contains("Duplicate entry") {
            if error.contains(".email'") {
                return UserServiceError::DuplicateEmail;
            }
        }

        UserServiceError::Fail
    }

    pub fn create(&self, data: User) -> Result<(), UserServiceError> {
        // if data.created_at.is_none() {
        //     data.created_at = Some(now_date_time_str());
        // }
        // if data.updated_at.is_none() {
        //     data.updated_at = Some(now_date_time_str());
        // }
        let items = vec![data];
        self.user_repository
            .get_ref()
            .insert(&items, None)
            .map_err(|e| self.match_error(e))
    }

    pub fn update(
        &self,
        data: &User,
        columns: &Option<Vec<UserColumn>>,
    ) -> Result<(), UserServiceError> {
        // if data.created_at.is_none() {
        //     data.created_at = Some(now_date_time_str());
        // }
        let filters = vec![UserFilter::Id(data.id)];
        // data.updated_at = Some(now_date_time_str());
        self.user_repository
            .get_ref()
            .update(&filters, &data, columns)
            .map_err(|e| self.match_error(e))
    }

    pub fn upsert(
        &self,
        data: User,
        columns: &Option<Vec<UserColumn>>,
    ) -> Result<(), UserServiceError> {
        if data.id == 0 {
            self.create(data)
        } else {
            self.update(&data, columns)
        }
    }

    pub fn update_password_by_id(&self, id: u64, password: &str) -> Result<(), UserServiceError> {
        let hash_service = self.hash_service.get_ref();
        let password = hash_service.hash_password(password).map_err(|e| {
            log::error!("UserService::update_password_by_id - {e}");
            UserServiceError::PasswordHashFail
        })?;

        self.user_repository
            .get_ref()
            .update_password_by_id(id, &password)
            .map_err(|e| self.match_error(e))
    }

    pub fn update_password_by_email(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(), UserServiceError> {
        let hash_service = self.hash_service.get_ref();
        let hashed_password = hash_service.hash_password(password).map_err(|e| {
            log::error!("UserService::update_password_by_email - {e}");
            UserServiceError::PasswordHashFail
        })?;

        self.user_repository
            .get_ref()
            .update_password_by_email(email, &hashed_password)
            .map_err(|e| self.match_error(e))
    }

    pub fn delete_by_id(&self, id: u64) -> Result<(), UserServiceError> {
        self.user_repository
            .get_ref()
            .delete_by_id(id)
            .map_err(|e| self.match_error(e))
    }

    pub fn delete_by_id_throw_http(&self, id: u64) -> Result<(), Error> {
        self.delete_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), UserServiceError> {
        self.user_repository
            .get_ref()
            .delete_by_ids(ids)
            .map_err(|e| self.match_error(e))
    }

    pub fn delete_by_ids_throw_http(&self, ids: &Vec<u64>) -> Result<(), Error> {
        self.delete_by_ids(ids)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn paginate(
        &self,
        params: &UserPaginateParams,
    ) -> Result<PaginationResult<User>, UserServiceError> {
        self.user_repository
            .get_ref()
            .paginate(params)
            .map_err(|e| self.match_error(e))
    }

    pub fn paginate_throw_http(
        &self,
        params: &UserPaginateParams,
    ) -> Result<PaginationResult<User>, Error> {
        self.paginate(params)
            .map_err(|_| error::ErrorInternalServerError(""))
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

impl TranslatableError for UserServiceError {
    fn translate(&self, lang: &str, translator_service: &TranslatorService) -> String {
        match self {
            Self::DbConnectionFail => {
                translator_service.translate(lang, "error.UserServiceError.DbConnectionFail")
            }
            Self::DuplicateEmail => {
                translator_service.translate(lang, "error.UserServiceError.DuplicateEmail")
            }
            Self::PasswordHashFail => {
                translator_service.translate(lang, "error.UserServiceError.PasswordHashFail")
            }
            Self::NotFound => translator_service.translate(lang, "error.UserServiceError.NotFound"),
            _ => translator_service.translate(lang, "error.UserServiceError.Fail"),
        }
    }
}
