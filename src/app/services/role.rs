use crate::{
    AppError, MysqlRepository, PaginationResult, Role, RoleColumn, RoleMysqlRepository,
    RolePaginateParams, TranslatableError, TranslatorService,
};
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

    pub fn get_all_ids(&self) -> Result<Vec<u64>, RoleServiceError> {
        self.role_repository
            .get_ref()
            .get_all_ids()
            .map_err(|e| self.match_error(e))
    }

    pub fn get_all_ids_throw_http(&self) -> Result<Vec<u64>, Error> {
        self.get_all_ids()
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn get_all(&self) -> Result<Vec<Role>, RoleServiceError> {
        self.role_repository
            .get_ref()
            .get_all()
            .map_err(|e| self.match_error(e))
    }

    pub fn get_all_throw_http(&self) -> Result<Vec<Role>, Error> {
        self.get_all()
            .map_err(|_| error::ErrorInternalServerError(""))
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

    fn match_error(&self, e: AppError) -> RoleServiceError {
        let error = e.to_string();

        if error.contains("Duplicate entry") {
            if error.contains(".code'") {
                return RoleServiceError::DuplicateCode;
            }
        }

        RoleServiceError::Fail
    }

    pub fn create(&self, data: &mut Role) -> Result<(), RoleServiceError> {
        self.role_repository
            .get_ref()
            .insert_one(data)
            .map_err(|e| self.match_error(e))
    }

    pub fn update(
        &self,
        data: &mut Role,
        columns: &Option<Vec<RoleColumn>>,
    ) -> Result<(), RoleServiceError> {
        self.role_repository
            .get_ref()
            .update_one(data, columns)
            .map_err(|e| self.match_error(e))
    }

    pub fn upsert(
        &self,
        data: &mut Role,
        columns: &Option<Vec<RoleColumn>>,
    ) -> Result<(), RoleServiceError> {
        if data.id == 0 {
            self.role_repository
                .get_ref()
                .insert_one(data)
                .map_err(|e| self.match_error(e))
        } else {
            self.role_repository
                .get_ref()
                .update_one(data, columns)
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
        params: &RolePaginateParams,
    ) -> Result<PaginationResult<Role>, RoleServiceError> {
        self.role_repository
            .get_ref()
            .paginate(params)
            .map_err(|e| self.match_error(e))
    }

    pub fn paginate_throw_http(
        &self,
        params: &RolePaginateParams,
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
    fn translate(&self, lang: &str, translator_service: &TranslatorService) -> String {
        match self {
            Self::DbConnectionFail => {
                translator_service.translate(lang, "error.RoleServiceError.DbConnectionFail")
            }
            Self::DuplicateCode => {
                translator_service.translate(lang, "error.RoleServiceError.DuplicateCode")
            }
            Self::NotFound => translator_service.translate(lang, "error.RoleServiceError.NotFound"),
            _ => translator_service.translate(lang, "error.RoleServiceError.Fail"),
        }
    }
}
