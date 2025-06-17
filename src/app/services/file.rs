use crate::{
    AppError, Disk, File, FileColumn, FileMysqlRepository, FilePaginateParams, MysqlRepository,
    PaginationResult, TranslatableError, TranslatorService,
};
use actix_web::web::Data;
use actix_web::{error, Error};
use strum_macros::{Display, EnumString};

pub struct FileService {
    file_repository: Data<FileMysqlRepository>,
}

impl FileService {
    pub fn new(file_repository: Data<FileMysqlRepository>) -> Self {
        Self { file_repository }
    }

    pub fn first_by_id(&self, file_id: u64) -> Result<Option<File>, FileServiceError> {
        self.file_repository
            .get_ref()
            .first_by_id(file_id)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_id_throw_http(&self, file_id: u64) -> Result<File, Error> {
        let user = self
            .first_by_id(file_id)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(user) = user {
            return Ok(user);
        }
        Err(error::ErrorNotFound(""))
    }

    pub fn first_by_local_path(
        &self,
        disk: &Disk,
        local_path: &str,
    ) -> Result<Option<File>, FileServiceError> {
        self.file_repository
            .get_ref()
            .first_by_local_path(disk, local_path)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_local_path_throw_http(
        &self,
        disk: &Disk,
        local_path: &str,
    ) -> Result<File, Error> {
        let user = self
            .first_by_local_path(disk, local_path)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(user) = user {
            return Ok(user);
        }
        Err(error::ErrorNotFound(""))
    }

    fn match_error(&self, e: AppError) -> FileServiceError {
        let error = e.to_string();

        if error.contains("Duplicate entry") {
            if error.contains(".local_path") {
                return FileServiceError::DuplicateLocalPath;
            }
        }

        FileServiceError::Fail
    }

    pub fn create(&self, data: &mut File) -> Result<(), FileServiceError> {
        self.file_repository
            .get_ref()
            .insert_one(data)
            .map_err(|e| self.match_error(e))
    }

    pub fn update(
        &self,
        data: &mut File,
        columns: &Option<Vec<FileColumn>>,
    ) -> Result<(), FileServiceError> {
        self.file_repository
            .get_ref()
            .update_one(data, columns)
            .map_err(|e| self.match_error(e))
    }

    pub fn upsert(
        &self,
        data: &mut File,
        columns: &Option<Vec<FileColumn>>,
    ) -> Result<(), FileServiceError> {
        if data.id == 0 {
            self.file_repository
                .get_ref()
                .insert_one(data)
                .map_err(|e| self.match_error(e))
        } else {
            self.file_repository
                .get_ref()
                .update_one(data, columns)
                .map_err(|e| self.match_error(e))
        }
    }

    pub fn delete_by_id(&self, id: u64) -> Result<(), FileServiceError> {
        self.file_repository
            .get_ref()
            .delete_by_id(id)
            .map_err(|e| self.match_error(e))
    }

    pub fn delete_by_id_throw_http(&self, id: u64) -> Result<(), Error> {
        self.delete_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), FileServiceError> {
        self.file_repository
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
        params: &FilePaginateParams,
    ) -> Result<PaginationResult<File>, FileServiceError> {
        self.file_repository
            .get_ref()
            .paginate(params)
            .map_err(|e| self.match_error(e))
    }

    pub fn paginate_throw_http(
        &self,
        params: &FilePaginateParams,
    ) -> Result<PaginationResult<File>, Error> {
        self.paginate(params)
            .map_err(|_| error::ErrorInternalServerError(""))
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString, PartialEq, Eq)]
pub enum FileServiceError {
    DbConnectionFail,
    DuplicateLocalPath,
    NotFound,
    Fail,
}

impl TranslatableError for FileServiceError {
    fn translate(&self, lang: &str, translator_service: &TranslatorService) -> String {
        match self {
            Self::DbConnectionFail => {
                translator_service.translate(lang, "error.FileServiceError.DbConnectionFail")
            }
            Self::DuplicateLocalPath => {
                translator_service.translate(lang, "error.FileServiceError.DuplicateLocalPath")
            }
            Self::NotFound => translator_service.translate(lang, "error.FileServiceError.NotFound"),
            _ => translator_service.translate(lang, "error.FileServiceError.Fail"),
        }
    }
}
