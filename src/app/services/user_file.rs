use crate::helpers::now_date_time_str;
use crate::{
    AppError, Config, Disk, DiskLocalRepository, File, FileServiceError, MysqlRepository,
    TranslatableError, TranslatorService, User, UserColumn, UserFile, UserFileColumn,
    UserFileFilter, UserFileMysqlRepository, UserFileSort, UserFilter, UserServiceError,
};
use actix_web::web::Data;
use actix_web::{error, Error};
use strum_macros::{Display, EnumString};

pub struct UserFileService {
    config: Config,
    user_file_repository: Data<UserFileMysqlRepository>,
    disk_local_repository: Data<DiskLocalRepository>,
}

impl UserFileService {
    pub fn new(
        config: Config,
        user_file_repository: Data<UserFileMysqlRepository>,
        disk_local_repository: Data<DiskLocalRepository>,
    ) -> Self {
        Self {
            config,
            user_file_repository,
            disk_local_repository,
        }
    }

    pub fn get_service_name(&self) -> &str {
        "UserFileService"
    }

    pub fn log_error(
        &self,
        method: &str,
        error: String,
        e: UserFileServiceError,
    ) -> UserFileServiceError {
        let service_name = self.get_service_name().to_string();
        log::error!("{}::{} - {}", service_name, method, error);
        e
    }

    pub fn first_by_user_id_and_file_id(
        &self,
        user_id: u64,
        file_id: u64,
    ) -> Result<Option<UserFile>, UserFileServiceError> {
        self.user_file_repository
            .get_ref()
            .first_by_user_id_and_file_id(user_id, file_id)
            .map_err(|e| UserFileServiceError::Fail)
    }

    pub fn all(
        &self,
        filters: Option<&Vec<UserFileFilter>>,
        sorts: Option<&Vec<UserFileSort>>,
    ) -> Result<Vec<UserFile>, UserFileServiceError> {
        self.user_file_repository
            .get_ref()
            .all(filters, sorts, &None)
            .map_err(|e| UserFileServiceError::Fail)
    }

    pub fn first_by_id(&self, id: u64) -> Result<Option<UserFile>, UserFileServiceError> {
        let filters = vec![UserFileFilter::Id(id)];
        self.user_file_repository
            .get_ref()
            .first(&filters)
            .map_err(|e| UserFileServiceError::Fail)
    }

    pub fn first_by_id_throw_http(&self, id: u64) -> Result<UserFile, Error> {
        let entity = self
            .first_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(entity) = entity {
            return Ok(entity);
        }
        Err(error::ErrorNotFound(""))
    }

    pub fn create(&self, mut data: UserFile, file: &File) -> Result<(), UserFileServiceError> {
        if data.created_at.is_none() {
            data.created_at = Some(now_date_time_str());
        }
        if data.updated_at.is_none() {
            data.updated_at = Some(now_date_time_str());
        }
        self.apply_is_public(&mut data, file)
            .map_err(|e| self.log_error("create", e.to_string(), UserFileServiceError::Fail))?;

        let items = vec![data];

        self.user_file_repository
            .get_ref()
            .insert(&items, None)
            .map_err(|e| UserFileServiceError::Fail)
    }

    pub fn update(
        &self,
        mut data: UserFile,
        columns: &Option<Vec<UserFileColumn>>,
        file: &File,
    ) -> Result<(), UserFileServiceError> {
        if data.created_at.is_none() {
            data.created_at = Some(now_date_time_str());
        }
        let filters = vec![UserFileFilter::Id(data.id)];
        data.updated_at = Some(now_date_time_str());

        self.apply_is_public(&mut data, file)
            .map_err(|e| self.log_error("update", e.to_string(), UserFileServiceError::Fail))?;

        self.user_file_repository
            .get_ref()
            .update(&filters, &data, columns)
            .map_err(|e| UserFileServiceError::Fail)
    }

    pub fn upsert(
        &self,
        data: UserFile,
        columns: &Option<Vec<UserFileColumn>>,
        file: &File,
    ) -> Result<(), UserFileServiceError> {
        if data.id == 0 {
            self.create(data, file)
        } else {
            self.update(data, columns, file)
        }
    }

    pub fn get_public_path(&self, user_file: &UserFile) -> Option<String> {
        if user_file.path.is_none() {
            return None;
        }

        if Disk::Local.to_string().eq(&user_file.disk) {
            if let Some(filename) = &user_file.filename {
                let mut public_path = self.config.filesystem.disks.local.url_path.to_owned();
                public_path.push('/');
                public_path.push_str(filename);
                return Some(public_path);
            }
        }
        None
    }

    pub fn get_public_url(&self, user_file: &UserFile) -> Option<String> {
        if let Some(public_path) = self.get_public_path(user_file) {
            if Disk::Local.to_string().eq(&user_file.disk) {
                let mut public_url = self.config.app.url.to_owned();
                public_url.push('/');
                public_url.push_str(&public_path);
                return Some(public_url);
            }
            Some(public_path)
        } else {
            None
        }
    }

    pub fn soft_delete_by_file_id(&self, file_id: u64) -> Result<(), UserFileServiceError> {
        self.user_file_repository
            .get_ref()
            .soft_delete_by_file_id(file_id)
            .map_err(|e| UserFileServiceError::Fail)
    }

    pub fn soft_delete_by_file_ids(&self, file_ids: &Vec<u64>) -> Result<(), UserFileServiceError> {
        self.user_file_repository
            .get_ref()
            .soft_delete_by_file_ids(file_ids)
            .map_err(|e| UserFileServiceError::Fail)
    }

    pub fn soft_delete_by_id(&self, id: u64) -> Result<(), UserFileServiceError> {
        self.user_file_repository
            .get_ref()
            .soft_delete_by_id(id)
            .map_err(|e| UserFileServiceError::Fail)
    }

    pub fn soft_delete_by_id_throw_http(&self, id: u64) -> Result<(), Error> {
        self.soft_delete_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn restore_by_id(&self, id: u64) -> Result<(), UserFileServiceError> {
        self.user_file_repository
            .get_ref()
            .restore_by_id(id)
            .map_err(|e| UserFileServiceError::Fail)
    }

    pub fn restore_by_id_throw_http(&self, id: u64) -> Result<(), Error> {
        self.restore_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn make_public_filename(&self, user_id: u64, filename: &str) -> String {
        let mut str = user_id.to_string();
        str.push('-');
        str.push_str(filename);
        str
    }

    pub fn apply_is_public(
        &self,
        user_file: &mut UserFile,
        file: &File,
    ) -> Result<bool, UserFileServiceError> {
        if user_file.disk.ne(Disk::Local.to_string().as_str()) {
            return Err(UserFileServiceError::Fail);
        }
        let disk_local_repository = self.disk_local_repository.get_ref();
        let mut is_updated = false;
        let filename = Some(self.make_public_filename(user_file.user_id, &file.filename));

        let public_path = disk_local_repository
            .set_public(&file.path, user_file.is_public, filename.clone())
            .map_err(|e| {
                self.log_error("apply_is_public", e.to_string(), UserFileServiceError::Fail)
            })?;

        if user_file.is_public {
            if filename.ne(&user_file.filename) {
                user_file.filename = filename.to_owned();
                is_updated = true;
            }

            if user_file.path.ne(&public_path) {
                user_file.path = public_path;
                is_updated = true;
            }
        } else {
            if user_file.filename.is_some() {
                user_file.filename = None;
                is_updated = true;
            }
            if user_file.path.is_some() {
                user_file.path = None;
                is_updated = true;
            }
        }
        Ok(is_updated)
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString, PartialEq, Eq)]
pub enum UserFileServiceError {
    DbConnectionFail,
    NotFound,
    Fail,
}

impl TranslatableError for UserFileServiceError {
    fn translate(&self, lang: &str, translator_service: &TranslatorService) -> String {
        match self {
            Self::DbConnectionFail => {
                translator_service.translate(lang, "error.UserFileServiceError.DbConnectionFail")
            }
            Self::NotFound => {
                translator_service.translate(lang, "error.UserFileServiceError.NotFound")
            }
            _ => translator_service.translate(lang, "error.UserFileServiceError.Fail"),
        }
    }
}
