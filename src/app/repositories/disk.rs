use crate::helpers::{
    collect_directories_from_dir_into_str_vec, collect_files_from_dir_into_str_vec,
    create_dir_all_for_file,
};
use actix_web::ResponseError;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::process::Command;
use std::time::SystemTime;
use ureq::http::StatusCode;

const FUN_NOT_DEFINED_ERROR_MESSAGE: &'static str = "The function is not defined.";

// https://github.com/laravel/framework/blob/12.x/src/Illuminate/Contracts/Filesystem/Filesystem.php
// pub trait DiskRepository<S> {
pub trait DiskRepository {
    // pub trait DiskRepository {
    // Get the full path to the file that exists at the given relative path.
    fn path(&self, path: &str) -> io::Result<String>;
    // #[allow(unused_variables)]
    // fn public_path(&self, path: &str) -> io::Result<String> {
    //     Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    // }
    // #[allow(unused_variables)]
    // fn set_public(&self, path: &str, is_public: bool) -> io::Result<Option<String>> {
    //     Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    // }
    #[allow(unused_variables)]
    fn hash(&self, file_path: &str) -> io::Result<String> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Determine if a file exists.
    fn exists(&self, path: &str) -> io::Result<bool>;
    // Get the contents of a file.
    fn get(&self, path: &str) -> io::Result<Vec<u8>>;
    // Get a resource to read the file.
    // fn read_stream(&self, path: &str) -> io::Result<S>;
    // Store the uploaded file on the disk.
    #[allow(unused_variables)]
    fn put(&self, path: &str, content: Vec<u8>) -> io::Result<()> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // // Write a new file using a stream.
    // fn write_stream(&self, path: &str, resource: &str, options: &str) -> io::Result<S>;
    // // Get the visibility for the given path.
    // fn get_visibility(&self, path: &str) -> io::Result<String>;
    // // Set the visibility for the given path.
    // fn set_visibility(&self, path: &str, visibility: &str) -> io::Result<()>;
    // // Prepend to a file.
    // fn prepend(&self, path: &str, data: &str) -> io::Result<()>;
    // // Append to a file.
    // fn append(&self, path: &str, data: &str) -> io::Result<()>;
    #[allow(unused_variables)]
    fn delete(&self, paths: &str) -> io::Result<()> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Delete the file at a given path.
    #[allow(unused_variables)]
    fn delete_many(&self, paths: &Vec<String>) -> io::Result<()> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Copy a file to a new location. On success, the total number of bytes copied is returned and it is equal to the length of the to file as reported by metadata.
    #[allow(unused_variables)]
    fn copy(&self, from: &str, to: &str) -> io::Result<u64> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Move a file to a new location.
    #[allow(unused_variables)]
    fn mv(&self, from: &str, to: &str) -> io::Result<()> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Get the file size of a given file.
    #[allow(unused_variables)]
    fn size(&self, path: &str) -> io::Result<u64> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Get the file's last modification time.
    #[allow(unused_variables)]
    fn last_modified(&self, path: &str) -> io::Result<SystemTime> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Get an array of all files in a directory.
    #[allow(unused_variables)]
    fn files(&self, directory: &str, recursive: bool) -> io::Result<Vec<String>> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Get all of the directories within a given directory.
    #[allow(unused_variables)]
    fn directories(&self, directory: &str, recursive: bool) -> io::Result<Vec<String>> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Create a directory.
    #[allow(unused_variables)]
    fn make_directory(&self, path: &str) -> io::Result<()> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Recursively delete a directory.
    #[allow(unused_variables)]
    fn delete_directory(&self, directory: &str) -> io::Result<()> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
}

pub struct DiskLocalRepository {
    pub root: String,
    pub separator: String,
    pub public_root: String,
}

impl DiskLocalRepository {
    pub fn new(root: &str, public_root: &str, separator: &str) -> Self {
        let mut root = root.trim().to_string();
        if root.ends_with(separator) && root.len() > 1 {
            root = root[..root.len() - 1].to_string();
        }
        let mut public_root = public_root.trim().to_string();
        if public_root.ends_with(separator) && public_root.len() > 1 {
            public_root = public_root[..public_root.len() - 1].to_string();
        }
        Self {
            root,
            separator: separator.to_string(),
            public_root,
        }
    }
    fn public_path(&self, path: &str) -> io::Result<String> {
        if path.starts_with(&self.root) {
            let path = path.replace(&self.root, "");
            make_local_path(&path, &self.public_root, &self.separator)
        } else {
            make_local_path(path, &self.public_root, &self.separator)
        }
    }
    pub fn link(&self, original: &str, link: &str) -> io::Result<()> {
        let result = fs::hard_link(original, link);

        if let Err(e) = result {
            if e.kind().eq(&ErrorKind::NotFound) {
                create_dir_all_for_file(link, &self.separator)?;
            }
            fs::hard_link(original, link)?;
        }

        Ok(())
    }
    pub fn set_public(
        &self,
        path: &str,
        is_public: bool,
        new_filename: Option<String>,
    ) -> io::Result<Option<String>> {
        let public_path = if let Some(new_filename) = &new_filename {
            self.public_path(new_filename)?
        } else {
            self.public_path(path)?
        };
        delete_from_local_path(public_path.as_str())?;
        if is_public {
            self.link(path, &public_path)?;
            Ok(Some(public_path))
        } else {
            Ok(None)
        }
    }
}

fn make_local_path(path: &str, root: &str, separator: &str) -> io::Result<String> {
    let mut result = root.to_owned();
    let path = path.trim();

    if !path.starts_with(separator) {
        result.push_str(separator);
    }

    result.push_str(path);
    Ok(result)
}

fn delete_from_local_path(path: &str) -> io::Result<()> {
    if fs::exists(&path)? {
        if fs::metadata(&path)?.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

// impl DiskRepository<File> for DiskLocalRepository {
impl DiskRepository for DiskLocalRepository {
    fn path(&self, path: &str) -> io::Result<String> {
        make_local_path(path, &self.root, &self.separator)
    }
    fn hash(&self, file_path: &str) -> io::Result<String> {
        let hash = Command::new("sha256sum").args([file_path]).output()?.stdout;
        let hash =
            String::from_utf8(hash).map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
        let hash = hash.split_whitespace().next().unwrap_or("").to_string();
        Ok(hash)
    }
    fn exists(&self, path: &str) -> io::Result<bool> {
        fs::exists(path)
    }
    fn get(&self, path: &str) -> io::Result<Vec<u8>> {
        fs::read(path)
    }
    // fn read_stream(&self, path: &str) -> io::Result<BufReader<File>> {
    //     let file = File::open(path)?;
    //     Ok(BufReader::new(file))
    // }
    fn size(&self, path: &str) -> io::Result<u64> {
        Ok(fs::metadata(path)?.size())
    }
    fn last_modified(&self, path: &str) -> io::Result<SystemTime> {
        fs::metadata(path)?.modified()
    }
    fn copy(&self, from: &str, to: &str) -> io::Result<u64> {
        if fs::exists(&from).unwrap_or(false) {
            create_dir_all_for_file(&to, &self.separator)?;
        }
        fs::copy(&from, &to)
    }
    fn mv(&self, from: &str, to: &str) -> io::Result<()> {
        if fs::exists(&from).unwrap_or(false) {
            create_dir_all_for_file(&to, &self.separator)?;
        }
        fs::rename(&from, &to)
    }
    fn put(&self, path: &str, content: Vec<u8>) -> io::Result<()> {
        create_dir_all_for_file(&path, &self.separator)?;
        fs::write(&path, content)
    }
    fn delete(&self, path: &str) -> io::Result<()> {
        delete_from_local_path(self.public_path(path)?.as_str())?;
        delete_from_local_path(path)?;
        Ok(())
    }
    fn delete_many(&self, paths: &Vec<String>) -> io::Result<()> {
        for path in paths {
            self.delete(path)?;
        }
        Ok(())
    }
    fn files(&self, directory: &str, recursive: bool) -> io::Result<Vec<String>> {
        let mut result: Vec<String> = Vec::new();
        collect_files_from_dir_into_str_vec(&mut result, &directory, recursive)?;
        Ok(result)
    }
    fn directories(&self, directory: &str, recursive: bool) -> io::Result<Vec<String>> {
        let mut result: Vec<String> = Vec::new();
        collect_directories_from_dir_into_str_vec(&mut result, &directory, recursive)?;
        Ok(result)
    }
    fn make_directory(&self, path: &str) -> io::Result<()> {
        fs::create_dir_all(path)
    }
    fn delete_directory(&self, directory: &str) -> io::Result<()> {
        fs::remove_dir_all(directory)
    }
}

pub struct DiskExternalRepository {}

impl DiskExternalRepository {
    pub fn new() -> Self {
        Self {}
    }
}

// impl DiskRepository<Vec<u8>> for DiskExternalRepository {
impl DiskRepository for DiskExternalRepository {
    fn path(&self, url: &str) -> io::Result<String> {
        let url = url.trim();
        if !url.starts_with("http") {
            return Err(io::Error::other("Http protocol not found in the url."));
        }
        Ok(url.to_string())
    }
    fn exists(&self, url: &str) -> io::Result<bool> {
        if let Ok(response) = ureq::get(url).call() {
            let status_code = response.status();

            if status_code.eq(&StatusCode::OK) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn get(&self, url: &str) -> io::Result<Vec<u8>> {
        if let Ok(mut response) = ureq::get(url).call() {
            if let Ok(result) = response.body_mut().read_to_vec() {
                return Ok(result);
            }
        }
        Err(io::Error::new(
            ErrorKind::NotFound,
            ErrorKind::NotFound.to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::MAIN_SEPARATOR_STR;

    #[test]
    fn test_local_disk_call_path() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_path
        let repository = DiskLocalRepository::new("/", "/", "/");
        assert_eq!(&repository.root, "/");
        let repository = DiskLocalRepository::new("/app/", "/", "/");
        assert_eq!(&repository.root, "/app");
        let repository = DiskLocalRepository::new("/app", "/", "/");
        assert_eq!(&repository.root, "/app");
        assert_eq!(repository.path("/test").unwrap().as_str(), "/app/test");
        assert_eq!(repository.path("/test ").unwrap().as_str(), "/app/test");
        assert_eq!(repository.path(" /test ").unwrap().as_str(), "/app/test");
        assert_eq!(repository.path(" test ").unwrap().as_str(), "/app/test");
        assert_eq!(repository.path("test").unwrap().as_str(), "/app/test");

        let repository = DiskLocalRepository::new("C:\\", "C:\\", "\\");
        assert_eq!(&repository.root, "C:");
        let repository = DiskLocalRepository::new("C:\\app\\", "C:\\app\\", "\\");
        assert_eq!(&repository.root, "C:\\app");
        let repository = DiskLocalRepository::new("C:\\app", "C:\\app", "\\");
        assert_eq!(&repository.root, "C:\\app");
        assert_eq!(repository.path("\\test").unwrap().as_str(), "C:\\app\\test");
        assert_eq!(
            repository.path("\\test ").unwrap().as_str(),
            "C:\\app\\test"
        );
        assert_eq!(
            repository.path(" \\test ").unwrap().as_str(),
            "C:\\app\\test"
        );
        assert_eq!(repository.path(" test ").unwrap().as_str(), "C:\\app\\test");
        assert_eq!(repository.path("test").unwrap().as_str(), "C:\\app\\test");
        assert_eq!(
            repository.path("/test/").unwrap().as_str(),
            "C:\\app\\/test/"
        );
    }

    #[test]
    fn test_local_disk_call_hash() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_hash
        fs::write("/app/test_local_disk_call_hash.txt", "123").unwrap();
        let repository = DiskLocalRepository::new("/app", "/app", "/");
        let hash = repository
            .hash("/app/test_local_disk_call_hash.txt")
            .unwrap();
        assert_eq!(
            hash,
            "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3"
        );
        fs::remove_file("/app/test_local_disk_call_hash.txt").unwrap();
    }

    #[test]
    fn test_external_disk_call_path() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_external_disk_call_path
        let repository = DiskExternalRepository::new();
        let url = repository.path(".gitignore");
        assert!(url.is_err());
        let url = repository.path("http://localhost");
        assert!(url.is_ok());
        let url = repository.path("https://localhost");
        assert!(url.is_ok());
    }

    #[test]
    fn test_local_disk_call_exists() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_exists
        let root = env::current_dir().unwrap();
        let repository = DiskLocalRepository::new(
            root.to_str().unwrap(),
            root.to_str().unwrap(),
            MAIN_SEPARATOR_STR,
        );
        let exists_file_path = repository.path(".gitignore").unwrap();
        assert!(repository.exists(&exists_file_path).unwrap());
        let no_exists_file_path = repository.path(".git_ignore").unwrap();
        assert_eq!(repository.exists(&no_exists_file_path).unwrap(), false);
        let exists_dir_path = repository.path("src").unwrap();
        assert!(repository.exists(&exists_dir_path).unwrap());
        let no_exists_dir_path = repository.path("s_r_c").unwrap();
        assert_eq!(repository.exists(&no_exists_dir_path).unwrap(), false);
    }

    #[test]
    fn test_external_disk_call_exists() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_external_disk_call_exists
        let repository = DiskExternalRepository::new();
        let url = repository.path(".gitignore");
        assert!(url.is_err());
        let url = repository.path("https://www.google.com/").unwrap();
        assert!(repository.exists(&url).unwrap());
        let url = repository.path("https://www.go_test_ogle.com/").unwrap();
        assert_eq!(repository.exists(&url).unwrap(), false);
    }

    #[test]
    fn test_local_disk_call_get() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_get
        let root = env::current_dir().unwrap();
        let repository = DiskLocalRepository::new(
            root.to_str().unwrap(),
            root.to_str().unwrap(),
            MAIN_SEPARATOR_STR,
        );
        let path = repository.path(".gitignore").unwrap();
        let content = repository.get(&path).unwrap();
        let content_str = String::from_utf8(content).unwrap();

        assert_ne!(content_str.len(), 0);

        let path = repository.path(".git_ignore").unwrap();
        let error = repository.get(&path);

        assert!(error.is_err());
    }

    #[test]
    fn test_external_disk_call_get() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_external_disk_call_get
        let repository = DiskExternalRepository::new();
        let path = repository
            .path("https://jsonplaceholder.typicode.com/todos/1")
            .unwrap();
        let content = repository.get(&path).unwrap();
        let content_str = String::from_utf8(content).unwrap();

        assert_ne!(content_str.len(), 0);

        let path = repository
            .path("https://jsonpla_test_ceholder.typi_test_code.com/todos/1")
            .unwrap();
        let error = repository.get(&path);

        assert!(error.is_err());
    }

    #[test]
    fn test_local_disk_call_put() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_put
        let root = env::current_dir().unwrap();
        let repository = DiskLocalRepository::new(
            root.to_str().unwrap(),
            root.to_str().unwrap(),
            MAIN_SEPARATOR_STR,
        );
        let dir_path = repository.path("/test_local_disk_call_put").unwrap();
        let path = repository
            .path("/test_local_disk_call_put/test.txt")
            .unwrap();
        let data = Vec::from(b"Test data");
        repository.put(&path, data).unwrap();
        assert!(fs::exists(&path).unwrap());
        fs::remove_dir_all(&dir_path).unwrap();
    }

    #[test]
    fn test_local_disk_call_directories() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_directories
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, root, MAIN_SEPARATOR_STR);
        let test_dir = repository
            .path("/test_local_disk_call_directories")
            .unwrap();

        let mut paths: Vec<String> = Vec::new();
        paths.push(
            repository
                .path("/test_local_disk_call_directories/test1_1/test2_1")
                .unwrap(),
        );
        paths.push(
            repository
                .path("/test_local_disk_call_directories/test1_2/test2_2")
                .unwrap(),
        );
        paths.push(
            repository
                .path("/test_local_disk_call_directories/test1_3/test2_3/test1_1")
                .unwrap(),
        );
        paths.push(
            repository
                .path("/test_local_disk_call_directories/test1_3/test2_3/test1_1")
                .unwrap(),
        );

        for path in paths {
            fs::create_dir_all(&path).unwrap();
        }

        let mut directories = repository.directories(&test_dir, false).unwrap();
        directories.sort();
        assert_eq!(
            directories,
            Vec::from([
                format!("{}/test_local_disk_call_directories/test1_1", root),
                format!("{}/test_local_disk_call_directories/test1_2", root),
                format!("{}/test_local_disk_call_directories/test1_3", root),
            ])
        );
        let mut rec_directories = repository.directories(&test_dir, true).unwrap();
        rec_directories.sort();
        assert_eq!(
            rec_directories,
            Vec::from([
                format!("{}/test_local_disk_call_directories/test1_1", root),
                format!("{}/test_local_disk_call_directories/test1_1/test2_1", root),
                format!("{}/test_local_disk_call_directories/test1_2", root),
                format!("{}/test_local_disk_call_directories/test1_2/test2_2", root),
                format!("{}/test_local_disk_call_directories/test1_3", root),
                format!("{}/test_local_disk_call_directories/test1_3/test2_3", root),
                format!(
                    "{}/test_local_disk_call_directories/test1_3/test2_3/test1_1",
                    root
                ),
            ])
        );

        fs::remove_dir_all(&test_dir).unwrap();
    }

    //noinspection DuplicatedCode
    #[test]
    fn test_local_disk_call_files() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_files
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, root, MAIN_SEPARATOR_STR);
        let test_dir = repository.path("/test_local_disk_call_files").unwrap();

        let mut paths: Vec<String> = Vec::new();
        paths.push(repository.path("/test_local_disk_call_files").unwrap());
        paths.push(
            repository
                .path("/test_local_disk_call_files/test1_2/test2_2")
                .unwrap(),
        );

        for path in paths {
            fs::create_dir_all(&path).unwrap();
            let filename1 = format!("{}/{}", &path, "file1.txt");
            let filename2 = format!("{}/{}", &path, "file2.txt");
            fs::write(&filename1, "test").unwrap();
            fs::write(&filename2, "test").unwrap();
        }

        let mut files = repository.files(&test_dir, false).unwrap();
        files.sort();
        assert_eq!(
            files,
            Vec::from([
                format!("{}/test_local_disk_call_files/file1.txt", root),
                format!("{}/test_local_disk_call_files/file2.txt", root),
            ])
        );
        let mut rec_files = repository.files(&test_dir, true).unwrap();
        rec_files.sort();
        assert_eq!(
            rec_files,
            Vec::from([
                format!("{}/test_local_disk_call_files/file1.txt", root),
                format!("{}/test_local_disk_call_files/file2.txt", root),
                format!(
                    "{}/test_local_disk_call_files/test1_2/test2_2/file1.txt",
                    root
                ),
                format!(
                    "{}/test_local_disk_call_files/test1_2/test2_2/file2.txt",
                    root
                ),
            ])
        );

        fs::remove_dir_all(&test_dir).unwrap();
    }

    //noinspection DuplicatedCode
    #[test]
    fn test_local_disk_call_delete() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_delete
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, root, MAIN_SEPARATOR_STR);
        let test_dir1 = repository.path("/test_local_disk_call_delete1").unwrap();
        let test_dir2 = repository.path("/test_local_disk_call_delete2").unwrap();

        let mut del_paths: Vec<String> = Vec::new();
        del_paths.push(test_dir1.to_owned());
        del_paths.push(test_dir2.to_owned());

        let mut paths: Vec<String> = Vec::new();
        paths.push(
            repository
                .path("/test_local_disk_call_delete1/test1_1/test1_2")
                .unwrap(),
        );
        paths.push(
            repository
                .path("/test_local_disk_call_delete2/test1_2/test2_2")
                .unwrap(),
        );

        for path in paths {
            fs::create_dir_all(&path).unwrap();
            let filename1 = format!("{}/{}", &path, "file1.txt");
            let filename2 = format!("{}/{}", &path, "file2.txt");
            fs::write(&filename1, "test").unwrap();
            fs::write(&filename2, "test").unwrap();
        }

        assert!(fs::exists(&test_dir1).unwrap());
        assert!(fs::exists(&test_dir2).unwrap());

        repository.delete_many(&del_paths).unwrap();

        assert!(!fs::exists(&test_dir1).unwrap());
        assert!(!fs::exists(&test_dir2).unwrap());
    }

    #[test]
    fn test_local_disk_call_copy() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_copy
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, root, MAIN_SEPARATOR_STR);
        let dir_path = repository.path("/test_local_disk_call_copy").unwrap();
        let dir_path2 = repository.path("test_local_disk_call_copy/test").unwrap();
        let file1_path = repository
            .path("/test_local_disk_call_copy/test/test1.txt")
            .unwrap();
        let file2_path = repository
            .path("/test_local_disk_call_copy/test2/test2.txt")
            .unwrap();
        fs::create_dir_all(dir_path2).unwrap();
        fs::write(&file1_path, "Test data").unwrap();
        repository.copy(&file1_path, &file2_path).unwrap();
        assert!(fs::exists(&file2_path).unwrap());
        fs::remove_dir_all(&dir_path).unwrap();
    }

    #[test]
    fn test_local_disk_call_mv() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_mv
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, root, MAIN_SEPARATOR_STR);
        let dir_path = repository.path("/test_local_disk_call_mv").unwrap();
        let dir_path2 = repository.path("/test_local_disk_call_mv/test").unwrap();
        let file1_path = repository
            .path("/test_local_disk_call_mv/test/test1.txt")
            .unwrap();
        let file2_path = repository
            .path("/test_local_disk_call_mv/test2/test2.txt")
            .unwrap();
        fs::create_dir_all(&dir_path2).unwrap();
        fs::write(&file1_path, "Test data").unwrap();
        repository.mv(&file1_path, &file2_path).unwrap();
        assert!(fs::exists(&file2_path).unwrap());
        assert!(!fs::exists(&file1_path).unwrap());
        fs::remove_dir_all(&dir_path).unwrap();
    }

    #[test]
    fn test_local_disk_call_size() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_size
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, root, MAIN_SEPARATOR_STR);
        let dir_path = repository.path("/test_local_disk_call_size").unwrap();
        let file1_path = repository
            .path("/test_local_disk_call_size/test1.txt")
            .unwrap();
        fs::create_dir_all(&dir_path).unwrap();
        fs::write(&file1_path, "Test data").unwrap();
        let size = repository.size(&file1_path).unwrap();
        assert!(size > 0);
        fs::remove_dir_all(&dir_path).unwrap();
    }

    #[test]
    fn test_local_disk_call_last_modified() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_last_modified
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, root, MAIN_SEPARATOR_STR);
        let dir_path = repository
            .path("/test_local_disk_call_last_modified")
            .unwrap();
        let file1_path = repository
            .path("/test_local_disk_call_last_modified/test1.txt")
            .unwrap();
        fs::create_dir_all(&dir_path).unwrap();
        fs::write(&file1_path, "Test data").unwrap();
        let _ = repository.last_modified(&file1_path).unwrap();
        fs::remove_dir_all(&dir_path).unwrap();
    }
}
