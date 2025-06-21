use std::path::MAIN_SEPARATOR;
use crate::{
    AppError, CryptServiceError, Disk, DiskExternalRepository, DiskLocalRepository, DiskRepository,
    File, FileColumn, FileMysqlRepository, FilePaginateParams, HashService, MysqlRepository,
    PaginationResult, RandomService, TranslatableError, TranslatorService, UploadData,
};
use actix_web::web::Data;
use actix_web::{error, Error};
use mime::Mime;
use strum_macros::{Display, EnumString};
use crate::helpers::now_date_time_str;

pub const FILE_DEFAULT_IS_PUBLIC: bool = false;
pub const FILE_DIRECTORY: &'static str = "files";

pub struct FileService {
    file_repository: Data<FileMysqlRepository>,
    disk_local_repository: Data<DiskLocalRepository>,
    disk_external_repository: Data<DiskExternalRepository>,
}

impl FileService {
    pub fn new(
        file_repository: Data<FileMysqlRepository>,
        disk_local_repository: Data<DiskLocalRepository>,
        disk_external_repository: Data<DiskExternalRepository>,
    ) -> Self {
        Self {
            file_repository,
            disk_local_repository,
            disk_external_repository,
        }
    }

    pub fn get_repository_name(&self) -> &str {
        "FileService"
    }

    pub fn log_error(&self, method: &str, error: String, e: FileServiceError) -> FileServiceError {
        let mut result = self.get_repository_name().to_string();
        result.push_str("::");
        result.push_str(method);
        result.push_str(" - ");
        result.push_str(&error);
        log::error!("{}", result);
        e
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

    pub fn make_local_path(&self, filename: &str) -> String {
        let mut str = FILE_DIRECTORY.to_string();
        str.push(MAIN_SEPARATOR);
        str.push_str(filename);
        str
    }

    pub fn make_uuid_filename(&self, file_hash: &str, mime: &Option<String>) -> String {
        let mut filename = file_hash.to_string();

        if let Some(mime) = mime {
            if let Some(extension) = mime2ext::mime2ext(mime) {
                filename.push('.');
                filename.push_str(&extension);
            }
        }

        filename
    }

    pub fn get_disk_repository(&self, disk: &Disk) -> &dyn DiskRepository {
        match disk {
            Disk::Local => self.disk_local_repository.get_ref(),
            Disk::External => self.disk_external_repository.get_ref(),
        }
    }

    pub fn upload(&self, data: UploadData) -> Result<(), FileServiceError> {
        let file_repository = self.file_repository.get_ref();

        let path = data.path;
        let mut filename: Option<String> = data.filename;
        let mut size: Option<u64> = data.size;
        let is_public: bool = data.is_public.unwrap_or(FILE_DEFAULT_IS_PUBLIC);
        let from_disk = data.from_disk.unwrap_or(Disk::default());
        let to_disk = data.to_disk.unwrap_or(Disk::default());

        let from_disk_repository: &dyn DiskRepository = self.get_disk_repository(&from_disk);
        let to_disk_repository: &dyn DiskRepository = self.get_disk_repository(&to_disk);

        let from_disk_path = from_disk_repository.path(&path).map_err(|e| {
            self.log_error(
                "upload_from_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;

        let is_exists = from_disk_repository.exists(&from_disk_path).map_err(|e| {
            self.log_error(
                "upload_from_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;
        if !is_exists {
            return Err(FileServiceError::NotFound);
        }

        let hash = from_disk_repository.hash(&from_disk_path).map_err(|e| {
            self.log_error(
                "upload_from_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;

        dbg!(hash.len());

        let mut mime: Option<Mime> = data.mime;

        if mime.is_none() {
            mime = mime_guess::from_path(path).first();
        }

        let mut mime_str: Option<String> = None;

        if let Some(mime) = mime {
            mime_str = Some(mime.to_string());
        }

        dbg!(&mime_str);

        let uuid_filename = self.make_uuid_filename(&hash, &mime_str);

        if filename.is_none() {
            filename = Some(uuid_filename.to_owned());
        }

        dbg!(&uuid_filename);

        let uuid_path: String = self.make_local_path(&uuid_filename);
        let local_path: String = to_disk_repository.path(&uuid_path).map_err(|e| {
            self.log_error(
                "upload_from_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            )
        })?;
        let mut public_path: Option<String> = None;
        if is_public {
            let public_path_: String = to_disk_repository.public_path(&uuid_path).map_err(|e| {
                self.log_error(
                    "upload_from_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;
            public_path = Some(public_path_);
        }

        if size.is_none() {
            let s = from_disk_repository.size(&from_disk_path).map_err(|e| {
                self.log_error(
                    "upload_from_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;
            size = Some(s);
        }

        let mut file = File::default();
        file.name = filename;
        file.public_path = public_path;
        file.local_path = local_path.to_owned();
        file.mime = mime_str;
        file.hash = Some(hash);
        file.size = size;
        file.creator_user_id = data.creator_user_id;
        file.created_at = Some(now_date_time_str());
        file.updated_at = Some(now_date_time_str());
        file.is_public = is_public;
        file.disk = to_disk.to_string();

        dbg!(&file);

        if from_disk.eq(&Disk::Local) && to_disk.eq(&Disk::Local) {
            to_disk_repository.mv(&from_disk_path, &local_path).map_err(|e| {
                self.log_error(
                    "upload_from_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;
        } else {
            let content = from_disk_repository.get(&from_disk_path).map_err(|e| {
                self.log_error(
                    "upload_from_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;

            to_disk_repository.put(&local_path, content).map_err(|e| {
                self.log_error(
                    "upload_from_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;
        }

        let result = file_repository.insert_one(&file);

        if let Err(e) = result {
            let p = vec![local_path];
            to_disk_repository.delete(&p).map_err(|e| {
                self.log_error(
                    "upload_from_local_disk",
                    e.to_string(),
                    FileServiceError::Fail,
                )
            })?;
            return Err(self.log_error(
                "upload_from_local_disk",
                e.to_string(),
                FileServiceError::Fail,
            ));
        }

        Ok(())
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
