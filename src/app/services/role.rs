use crate::{AppError, MysqlRepository, PaginationResult, Role, RoleColumn, RoleFilter, RoleMysqlRepository, RolePaginateParams, TranslatableError, TranslatorService, User, UserColumn, UserFilter, UserServiceError};
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

    pub fn all(&self) -> Result<Vec<Role>, RoleServiceError> {
        self.role_repository
            .get_ref()
            .all(None, None, &None)
            .map_err(|e| self.match_error(e))
    }

    pub fn all_throw_http(&self) -> Result<Vec<Role>, Error> {
        self.all()
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn first_by_id(&self, id: u64) -> Result<Option<Role>, RoleServiceError> {
        self.role_repository
            .get_ref()
            .first_by_id(id)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_id_throw_http(&self, id: u64) -> Result<Role, Error> {
        let entity = self
            .first_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(entity) = entity {
            return Ok(entity);
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
        let entity = self
            .first_by_code(code)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(entity) = entity {
            return Ok(entity);
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

    pub fn create(&self, data: Role) -> Result<(), RoleServiceError> {
        // if data.created_at.is_none() {
        //     data.created_at = Some(now_date_time_str());
        // }
        // if data.updated_at.is_none() {
        //     data.updated_at = Some(now_date_time_str());
        // }
        let items = vec![data];
        self.role_repository
            .get_ref()
            .insert(&items, None)
            .map_err(|e| self.match_error(e))
    }

    pub fn update(
        &self,
        data: &Role,
        columns: &Option<Vec<RoleColumn>>,
    ) -> Result<(), RoleServiceError> {
        // if data.created_at.is_none() {
        //     data.created_at = Some(now_date_time_str());
        // }
        let filters = vec![RoleFilter::Id(data.id)];
        // data.updated_at = Some(now_date_time_str());
        self.role_repository
            .get_ref()
            .update(&filters, &data, columns)
            .map_err(|e| self.match_error(e))
    }

    pub fn upsert(
        &self,
        data: Role,
        columns: &Option<Vec<RoleColumn>>,
    ) -> Result<(), RoleServiceError> {
        if data.id == 0 {
            self.create(data)
        } else {
            self.update(&data, columns)
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
