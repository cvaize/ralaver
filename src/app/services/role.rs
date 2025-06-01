use crate::{PaginationResult, Role, RoleMysqlRepository, RoleMysqlRepositoryError, RoleMysqlRepositoryPaginateParams, TranslatableError, TranslatorService, UserServiceError};
use actix_web::web::Data;
use actix_web::{error, Error};
use strum_macros::{Display, EnumString};

pub struct RoleService {
    role_repository: Data<RoleMysqlRepository>,
}

impl RoleService {
    pub fn new(role_repository: Data<RoleMysqlRepository>) -> Self {
        Self { role_repository }
    }

    pub fn first_by_id(&self, role_id: u64) -> Result<Option<Role>, RoleServiceError> {
        self.role_repository
            .get_ref()
            .first_by_id(role_id)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_id_throw_http(&self, role_id: u64) -> Result<Role, Error> {
        let user = self
            .first_by_id(role_id)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(user) = user {
            return Ok(user);
        }
        Err(error::ErrorNotFound(""))
    }

    pub fn first_by_code(&self, code: &str) -> Result<Option<Role>, RoleServiceError> {
        self.role_repository
            .get_ref()
            .first_by_code(code)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_code_throw_http(&self, code: &str) -> Result<Role, Error> {
        let user = self
            .first_by_code(code)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(user) = user {
            return Ok(user);
        }
        Err(error::ErrorNotFound(""))
    }

    fn match_error(&self, e: RoleMysqlRepositoryError) -> RoleServiceError {
        match e {
            RoleMysqlRepositoryError::DuplicateCode => RoleServiceError::DuplicateCode,
            _ => RoleServiceError::Fail,
        }
    }

    pub fn create(&self, data: &mut Role) -> Result<(), RoleServiceError> {
        self.role_repository
            .get_ref()
            .insert(data)
            .map_err(|e| self.match_error(e))
    }

    pub fn update(&self, data: &mut Role) -> Result<(), RoleServiceError> {
        self.role_repository
            .get_ref()
            .update(data)
            .map_err(|e| self.match_error(e))
    }

    pub fn upsert(&self, data: &mut Role) -> Result<(), RoleServiceError> {
        if data.id == 0 {
            self.role_repository
                .get_ref()
                .insert(data)
                .map_err(|e| self.match_error(e))
        } else {
            self.role_repository
                .get_ref()
                .update(data)
                .map_err(|e| self.match_error(e))
        }
    }

    pub fn delete_by_id(&self, id: u64) -> Result<(), RoleServiceError> {
        self.role_repository
            .get_ref()
            .delete_by_id(id)
            .map_err(|e| self.match_error(e))
    }

    pub fn delete_by_id_throw_http(&self, id: u64) -> Result<(), Error> {
        self.delete_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), RoleServiceError> {
        self.role_repository
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
        params: &RoleMysqlRepositoryPaginateParams,
    ) -> Result<PaginationResult<Role>, RoleServiceError> {
        self.role_repository
            .get_ref()
            .paginate(params)
            .map_err(|e| self.match_error(e))
    }

    pub fn paginate_throw_http(
        &self,
        params: &RoleMysqlRepositoryPaginateParams,
    ) -> Result<PaginationResult<Role>, Error> {
        self.paginate(params)
            .map_err(|_| error::ErrorInternalServerError(""))
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString, PartialEq, Eq)]
pub enum RoleServiceError {
    DbConnectionFail,
    DuplicateCode,
    NotFound,
    Fail,
}

impl TranslatableError for RoleServiceError {
    fn translate(&self, lang: &str, translate_service: &TranslatorService) -> String {
        match self {
            Self::DbConnectionFail => {
                translate_service.translate(lang, "error.RoleServiceError.DbConnectionFail")
            }
            Self::DuplicateCode => {
                translate_service.translate(lang, "error.RoleServiceError.DuplicateCode")
            }
            Self::NotFound => {
                translate_service.translate(lang, "error.RoleServiceError.NotFound")
            }
            _ => translate_service.translate(lang, "error.RoleServiceError.Fail"),
        }
    }
}