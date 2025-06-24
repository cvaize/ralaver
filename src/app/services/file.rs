use crate::helpers::now_date_time_str;
use crate::{
    AppError, CryptServiceError, Disk, DiskExternalRepository, DiskLocalRepository, DiskRepository,
    File, FileColumn, FileMysqlRepository, FilePaginateParams, HashService, MysqlRepository,
    PaginationResult, RandomService, TranslatableError, TranslatorService, UploadData,
};
use actix_web::web::Data;
use actix_web::{error, Error};
use mime::Mime;
use std::fmt::format;
use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR};
use strum_macros::{Display, EnumString};

pub const FILE_DEFAULT_IS_PUBLIC: bool = false;
pub const FILE_DIRECTORY: &'static str = "files";

pub struct FileService {
    file_repository: Data<FileMysqlRepository>,
    disk_local_repository: Data<DiskLocalRepository>,
    disk_external_repository: Data<DiskExternalRepository>,
    random_repository: Data<RandomService>,
}

impl FileService {
    pub fn new(
        file_repository: Data<FileMysqlRepository>,
        disk_local_repository: Data<DiskLocalRepository>,
        disk_external_repository: Data<DiskExternalRepository>,
        random_repository: Data<RandomService>,
    ) -> Self {
        Self {
            file_repository,
            disk_local_repository,
            disk_external_repository,
            random_repository,
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

    pub fn first_by_disk_and_local_path(
        &self,
        disk: &Disk,
        local_path: &str,
    ) -> Result<Option<File>, FileServiceError> {
        self.file_repository
            .get_ref()
            .first_by_disk_and_local_path(disk, local_path)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_local_path_throw_http(
        &self,
        disk: &Disk,
        local_path: &str,
    ) -> Result<File, Error> {
        let user = self
            .first_by_disk_and_local_path(disk, local_path)
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

    pub fn generate_uuid_local_path(
        &self,
        mime: &Option<String>,
        disk_repository: &dyn DiskRepository,
    ) -> Result<(String, String), FileServiceError> {
        let random_repository = self.random_repository.get_ref();
        let mut extension: Option<&str> = None;
        if let Some(mime) = mime {
            extension = mime2ext::mime2ext(mime);
        }

        let mut uuid_filename: Option<String> = None;
        let mut local_path: Option<String> = None;
        for _ in 0..100 {
            let mut uuid_filename_: String = random_repository.str(32);

            if let Some(extension) = extension {
                uuid_filename_.push('.');
                uuid_filename_.push_str(&extension);
            }

            let local_path_: String = disk_repository
                .path(self.make_local_path(&uuid_filename_).as_str())
                .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;
            let is: bool = disk_repository
                .exists(&local_path_)
                .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;
            if !is {
                uuid_filename = Some(uuid_filename_);
                local_path = Some(local_path_);
                break;
            }
        }

        if local_path.is_none() {
            return Err(self.log_error(
                "upload",
                "Couldn't find the path for the new file".to_string(),
                FileServiceError::NotFound,
            ));
        }

        let uuid_filename: String = uuid_filename.unwrap();
        let local_path: String = local_path.unwrap();
        Ok((uuid_filename, local_path))
    }

    pub fn get_disk_repository(&self, disk: &Disk) -> &dyn DiskRepository {
        match disk {
            Disk::Local => self.disk_local_repository.get_ref(),
            Disk::External => self.disk_external_repository.get_ref(),
        }
    }

    fn prepare_filename(&self, mut filename: Option<String>, mime: &Option<Mime>) -> Option<String> {
        if let Some(filename_) = &filename {
            let filename__ = filename_.trim();
            if filename__.len() != filename_.len() {
                filename = Some(filename__.to_string());
            }
        }

        if let Some(mime) = &mime {
            if let Some(filename_) = &filename {
                let g_mime: Option<Mime> = mime_guess::from_path(filename_.to_lowercase()).first();
                let mut is = g_mime.is_none();
                if let Some(g_mime) = g_mime {
                    if mime.ne(&g_mime) {
                        is = true;
                    }
                }

                if is {
                    let mime_s = mime.to_string();
                    if let Some(extension) = mime2ext::mime2ext(&mime_s) {
                        let mut filename_ = filename_.to_owned();
                        filename_.push('.');
                        filename_.push_str(extension);
                        filename = Some(filename_);
                    }
                }
            }
        }

        filename
    }

    pub fn upload_external_file_to_external_disk(
        &self,
        url: &str,
        data: UploadData,
    ) -> Result<(), FileServiceError> {
        let disk_external_repository = self.disk_external_repository.get_ref();
        let file_repository = self.file_repository.get_ref();

        let mut id: u64 = 0;
        let local_path: String = url.to_string();
        let mut filename: Option<String> = data.filename;
        let mut mime: Option<Mime> = data.mime;
        let mut mime_str: Option<String> = if let Some(mime) = &mime {
            Some(mime.to_string())
        } else {
            None
        };
        let is_public: bool = data.is_public.unwrap_or(FILE_DEFAULT_IS_PUBLIC);
        let size: Option<u64> = data.size;
        let hash: Option<String> = data.hash;
        let disk: Disk = Disk::External;
        let mut creator_user_id: Option<u64> = data.creator_user_id;

        if filename.is_none() || mime_str.is_none() {
            for path in local_path.split("/") {
                if filename.is_some() && mime_str.is_some() {
                    break;
                }

                if path.contains(".") {
                    let g_mime: Option<Mime> = mime_guess::from_path(path).first();

                    if filename.is_none() && g_mime.is_some() {
                        filename = Some(path.trim().to_string());
                    }

                    if mime_str.is_none() && g_mime.is_some() {
                        if let Some(mime1) = g_mime {
                            mime_str = Some(mime1.to_string());
                            mime = Some(mime1);
                        }
                    }
                }
            }
        }

        filename = self.prepare_filename(filename, &mime);

        let old_file = file_repository
            .first_by_disk_and_local_path(&disk, &local_path)
            .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;

        if let Some(old_file) = old_file {
            id = old_file.id;
            if let Some(old_creator_user_id) = old_file.creator_user_id {
                creator_user_id = Some(old_creator_user_id);
            }
        }

        let mut file = File {
            id,
            filename,
            public_path: None,
            local_path,
            mime: mime_str,
            hash,
            size,
            creator_user_id,
            created_at: Some(now_date_time_str()),
            updated_at: Some(now_date_time_str()),
            file_delete_at: None,
            file_deleted_at: None,
            deleted_at: None,
            is_deleted: false,
            is_public,
            disk: disk.to_string(),
        };

        self.upsert(&mut file, &None)
            .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;

        Ok(())
    }

    pub fn upload_local_file_to_local_disk(
        &self,
        path: &str,
        data: UploadData,
    ) -> Result<(), FileServiceError> {
        let file_repository = self.file_repository.get_ref();
        let disk_local_repository = self.disk_local_repository.get_ref();

        let mut filename: Option<String> = data.filename;
        let mut size: Option<u64> = None;
        let is_public: bool = data.is_public.unwrap_or(FILE_DEFAULT_IS_PUBLIC);
        let disk = Disk::Local;

        let is_exists = disk_local_repository
            .exists(path)
            .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;
        if !is_exists {
            return Err(self.log_error(
                "upload",
                format!("File not found {}", path),
                FileServiceError::NotFound,
            ));
        }

        let hash = disk_local_repository
            .hash(path)
            .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;

        let mime: Option<Mime> = if data.mime.is_none() {
            mime_guess::from_path(path).first()
        } else {
            data.mime
        };

        let mime_str: Option<String> = if let Some(mime) = &mime {
            Some(mime.to_string())
        } else {
            None
        };

        let mut uuid_filename = hash.to_owned();

        if let Some(mime_str) = &mime_str {
            if let Some(extension) = mime2ext::mime2ext(mime_str) {
                uuid_filename.push('.');
                uuid_filename.push_str(&extension);
            }
        }

        let uuid_path: String = self.make_local_path(&uuid_filename);
        let local_path: String = disk_local_repository
            .path(&uuid_path)
            .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;

        let is_exists_in_fs = disk_local_repository
            .exists(&local_path)
            .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;

        let old_file = file_repository
            .first_by_disk_and_local_path(&disk, &local_path)
            .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;

        let mut is_copy = true;

        if is_exists_in_fs {
            let exists_file_hash = disk_local_repository
                .hash(&local_path)
                .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;

            if exists_file_hash.ne(&hash) {
                disk_local_repository
                    .delete(&local_path)
                    .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;
            } else {
                is_copy = false;
            }
        }

        let s = disk_local_repository
            .size(path)
            .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;
        size = Some(s);

        if is_copy {
            let content = disk_local_repository
                .get(path)
                .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;
            disk_local_repository
                .put(&local_path, content)
                .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;
        }

        filename = self.prepare_filename(filename, &mime);

        if filename.is_none() {
            let path_ = path.split(MAIN_SEPARATOR_STR).last();
            if let Some(path__) = path_ {
                let path__ = path__.trim();
                if !path__.is_empty() {
                    filename = Some(path__.to_string());
                }
            }
        }

        if filename.is_none() {
            filename = Some(uuid_filename.to_owned());
        }

        let mut file = File::default();
        file.filename = filename;
        file.local_path = local_path.to_owned();
        file.mime = mime_str;
        file.hash = Some(hash);
        file.size = size;
        file.creator_user_id = data.creator_user_id;
        file.created_at = Some(now_date_time_str());
        file.updated_at = Some(now_date_time_str());
        file.is_public = is_public;
        file.disk = disk.to_string();

        if let Some(old_file) = old_file {
            file.id = old_file.id;
        }

        let result = self.upsert(&mut file, &None);

        if let Err(e) = result {
            disk_local_repository
                .delete(&local_path)
                .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;
            file_repository
                .delete_by_disk_and_local_path(&disk, &local_path)
                .map_err(|e| self.log_error("upload", e.to_string(), FileServiceError::Fail))?;
            return Err(self.log_error("upload", e.to_string(), FileServiceError::Fail));
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

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::MAIN_SEPARATOR_STR;
    use crate::{preparation, Disk, UploadData};
    use test::Bencher;

    #[test]
    fn test_upload_external_file_to_external_disk() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::services::file::tests::test_upload_external_file_to_external_disk
        let (_, all_services) = preparation();
        let file_service = all_services.file_service.get_ref();

        let disk = Disk::External;
        let creator_user_id = Some(1);
        let url = "http://localhost/image.jpg/orig";
        let data = UploadData {
            mime: Some(mime::IMAGE_JPEG.to_owned()),
            filename: Some("test_image.jpeg".to_string()),
            size: Some(22),
            is_public: Some(true),
            hash: Some("test_hash".to_string()),
            creator_user_id: creator_user_id.clone(),
        };
        file_service
            .upload_external_file_to_external_disk(url, data.clone())
            .unwrap();

        let file = file_service
            .first_by_disk_and_local_path(&disk, url)
            .unwrap()
            .unwrap();

        assert_eq!(&file.local_path, url);
        assert_eq!(file.mime.unwrap(), data.mime.unwrap().to_string());
        assert_eq!(&file.filename, &Some("test_image.jpeg".to_string()));
        assert_eq!(&file.size, &data.size);
        assert_eq!(file.is_public, data.is_public.unwrap());
        assert_eq!(&file.hash, &data.hash);
        assert_eq!(&file.creator_user_id, &data.creator_user_id);

        let data = UploadData {
            mime: Some(mime::IMAGE_JPEG.to_owned()),
            filename: Some("test_image.txt".to_string()),
            size: Some(22),
            is_public: Some(true),
            hash: Some("test_hash".to_string()),
            creator_user_id: creator_user_id.clone(),
        };
        file_service
            .upload_external_file_to_external_disk(url, data.clone())
            .unwrap();

        let file = file_service
            .first_by_disk_and_local_path(&disk, url)
            .unwrap()
            .unwrap();

        assert_eq!(&file.local_path, url);
        assert_eq!(file.mime.unwrap(), data.mime.unwrap().to_string());
        assert_eq!(&file.filename, &Some("test_image.txt.jpg".to_string()));
        assert_eq!(&file.size, &data.size);
        assert_eq!(file.is_public, data.is_public.unwrap());
        assert_eq!(&file.hash, &data.hash);
        assert_eq!(&file.creator_user_id, &data.creator_user_id);

        let data = UploadData {
            mime: Some(mime::IMAGE_JPEG.to_owned()),
            filename: Some("test_image_2".to_string()),
            size: Some(22),
            is_public: Some(true),
            hash: Some("test_hash".to_string()),
            creator_user_id: creator_user_id.clone(),
        };
        file_service
            .upload_external_file_to_external_disk(url, data.clone())
            .unwrap();

        let file = file_service
            .first_by_disk_and_local_path(&disk, url)
            .unwrap()
            .unwrap();

        assert_eq!(&file.local_path, url);
        assert_eq!(file.mime.unwrap(), data.mime.unwrap().to_string());
        assert_eq!(&file.filename, &Some("test_image_2.jpg".to_string()));
        assert_eq!(&file.size, &data.size);
        assert_eq!(file.is_public, data.is_public.unwrap());
        assert_eq!(&file.hash, &data.hash);
        assert_eq!(&file.creator_user_id, &data.creator_user_id);

        let data = UploadData::default();
        file_service
            .upload_external_file_to_external_disk(url, data.clone())
            .unwrap();

        let file = file_service
            .first_by_disk_and_local_path(&disk, url)
            .unwrap()
            .unwrap();

        assert_eq!(&file.local_path, url);
        assert_eq!(file.mime.unwrap(), mime::IMAGE_JPEG.to_string());
        assert_eq!(&file.filename, &Some("image.jpg".to_string()));
        assert_eq!(&file.size, &data.size);
        assert_eq!(file.is_public, false);
        assert_eq!(&file.hash, &data.hash);
        assert_eq!(&file.creator_user_id, &creator_user_id);

        file_service.delete_by_id(file.id).unwrap();
    }

    #[test]
    fn test_upload_local_file_to_local_disk() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::services::file::tests::test_upload_local_file_to_local_disk
        let (_, all_services) = preparation();
        let file_service = all_services.file_service.get_ref();

        let root = env::current_dir().unwrap();
        let root_dir = root.to_str().unwrap();

        let disk = Disk::Local;
        let creator_user_id = Some(1);

        let mut path = root_dir.to_string();
        if !path.ends_with(MAIN_SEPARATOR_STR) {
            path.push_str(MAIN_SEPARATOR_STR);
        }
        path.push_str("Readme.md");

        let data = UploadData {
            mime: None,
            filename: Some("Readme.md".to_string()),
            size: None,
            is_public: Some(false),
            hash: None,
            creator_user_id: creator_user_id.clone(),
        };
        file_service
            .upload_local_file_to_local_disk(&path, data.clone())
            .unwrap();

        // let file = file_service
        //     .first_by_disk_and_local_path(&disk, &path)
        //     .unwrap()
        //     .unwrap();
        //
        // dbg!(&file);
    }

    // #[bench]
    // fn bench_encrypt_string(b: &mut Bencher) {
    //     let (_, all_services) = preparation();
    //     let crypt = all_services.crypt_service.get_ref();
    //
    //     b.iter(|| {
    //         let _ = crypt.encrypt_string(DATA).unwrap();
    //     })
    // }
}
