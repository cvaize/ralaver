use crate::helpers::now_date_time_str;
use crate::{
    Config, Disk, MysqlRepository, TranslatableError, TranslatorService, UserFile, UserFileColumn,
    UserFileFilter, UserFileMysqlRepository, UserFileSort,
};
use actix_web::web::Data;
use actix_web::{error, Error};
use strum_macros::{Display, EnumString};

pub struct UserFileService {
    config: Data<Config>,
    user_file_repository: Data<UserFileMysqlRepository>,
}

impl UserFileService {
    pub fn new(config: Data<Config>, user_file_repository: Data<UserFileMysqlRepository>) -> Self {
        Self {
            config,
            user_file_repository,
        }
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

    pub fn upsert(
        &self,
        mut data: UserFile,
        columns: &Option<Vec<UserFileColumn>>,
    ) -> Result<(), UserFileServiceError> {
        if data.created_at.is_none() {
            data.created_at = Some(now_date_time_str());
        }
        if data.id == 0 {
            if data.updated_at.is_none() {
                data.updated_at = Some(now_date_time_str());
            }
            let items = vec![data];
            self.user_file_repository
                .get_ref()
                .insert(&items, None)
                .map_err(|e| UserFileServiceError::Fail)
        } else {
            let filters = vec![UserFileFilter::Id(data.id)];
            data.updated_at = Some(now_date_time_str());
            self.user_file_repository
                .get_ref()
                .update(&filters, &data, columns)
                .map_err(|e| UserFileServiceError::Fail)
        }
    }

    pub fn get_public_path(&self, user_file: &UserFile) -> Option<String> {
        if user_file.path.is_none() {
            return None;
        }

        if Disk::Local.to_string().eq(&user_file.disk) {
            if let Some(filename) = &user_file.filename {
                let config = self.config.get_ref();
                let mut public_path = config.filesystem.disks.local.url_path.to_owned();
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
                let config = self.config.get_ref();
                let mut public_url = config.app.url.to_owned();
                public_url.push('/');
                public_url.push_str(&public_path);
                return Some(public_url);
            }
            Some(public_path)
        } else {
            None
        }
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
