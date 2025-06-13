use crate::helpers::{
    collect_directories_from_dir_into_str_vec, collect_files_from_dir_into_str_vec,
    create_dir_all_for_file,
};
use futures_core::Stream;
use reqwest::StatusCode;
use std::fs;
use std::io;
use std::io::{ErrorKind, Read};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::SystemTime;

const FUN_NOT_DEFINED_ERROR_MESSAGE: &'static str = "The function is not defined.";

// https://github.com/laravel/framework/blob/12.x/src/Illuminate/Contracts/Filesystem/Filesystem.php
// pub trait DiskRepository<S> {
pub trait DiskRepository {
    // pub trait DiskRepository {
    // Get the full path to the file that exists at the given relative path.
    async fn path(&self, path: &str) -> io::Result<String>;
    // Determine if a file exists.
    async fn exists(&self, path: &str) -> io::Result<bool>;
    // Get the contents of a file.
    async fn get(&self, path: &str) -> io::Result<Vec<u8>>;
    // Get a resource to read the file.
    // async fn read_stream(&self, path: &str) -> io::Result<S>;
    // Store the uploaded file on the disk.
    async fn put(&self, path: &str, content: Vec<u8>) -> io::Result<()> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // // Write a new file using a stream.
    // async fn write_stream(&self, path: &str, resource: &str, options: &str) -> io::Result<S>;
    // // Get the visibility for the given path.
    // async fn get_visibility(&self, path: &str) -> io::Result<String>;
    // // Set the visibility for the given path.
    // async fn set_visibility(&self, path: &str, visibility: &str) -> io::Result<()>;
    // // Prepend to a file.
    // async fn prepend(&self, path: &str, data: &str) -> io::Result<()>;
    // // Append to a file.
    // async fn append(&self, path: &str, data: &str) -> io::Result<()>;
    // Delete the file at a given path.
    async fn delete(&self, paths: &Vec<String>) -> io::Result<()> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Copy a file to a new location. On success, the total number of bytes copied is returned and it is equal to the length of the to file as reported by metadata.
    async fn copy(&self, from: &str, to: &str) -> io::Result<u64> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Move a file to a new location.
    async fn mv(&self, from: &str, to: &str) -> io::Result<()> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Get the file size of a given file.
    async fn size(&self, path: &str) -> io::Result<u64> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Get the file's last modification time.
    async fn last_modified(&self, path: &str) -> io::Result<SystemTime> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Get an array of all files in a directory.
    async fn files(&self, directory: &str, recursive: bool) -> io::Result<Vec<String>> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Get all of the directories within a given directory.
    async fn directories(&self, directory: &str, recursive: bool) -> io::Result<Vec<String>> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Create a directory.
    async fn make_directory(&self, path: &str) -> io::Result<()> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
    // Recursively delete a directory.
    async fn delete_directory(&self, directory: &str) -> io::Result<()> {
        Err(io::Error::other(FUN_NOT_DEFINED_ERROR_MESSAGE))
    }
}

pub struct DiskLocalRepository {
    pub root: String,
    pub separator: String,
}

impl DiskLocalRepository {
    pub fn new(root: &str, separator: &str) -> Self {
        let mut root = root.trim().to_string();
        if root.ends_with(separator) && root.len() > 1 {
            root = root[..root.len() - 1].to_string();
        }
        Self {
            root,
            separator: separator.to_string(),
        }
    }
}

// impl DiskRepository<File> for DiskLocalRepository {
impl DiskRepository for DiskLocalRepository {
    async fn path(&self, path: &str) -> io::Result<String> {
        let mut result = self.root.to_owned();
        let path = path.trim();

        if !path.starts_with(&self.separator) {
            result.push_str(&self.separator);
        }

        result.push_str(path);
        Ok(result)
    }
    async fn exists(&self, path: &str) -> io::Result<bool> {
        fs::exists(path)
    }
    async fn get(&self, path: &str) -> io::Result<Vec<u8>> {
        fs::read(path)
    }
    // async fn read_stream(&self, path: &str) -> io::Result<BufReader<File>> {
    //     let file = File::open(path)?;
    //     Ok(BufReader::new(file))
    // }
    async fn size(&self, path: &str) -> io::Result<u64> {
        Ok(fs::metadata(path)?.size())
    }
    async fn last_modified(&self, path: &str) -> io::Result<SystemTime> {
        fs::metadata(path)?.modified()
    }
    async fn copy(&self, from: &str, to: &str) -> io::Result<u64> {
        if fs::exists(from).unwrap_or(false) {
            create_dir_all_for_file(to, &self.separator)?;
        }
        fs::copy(from, to)
    }
    async fn mv(&self, from: &str, to: &str) -> io::Result<()> {
        if fs::exists(from).unwrap_or(false) {
            create_dir_all_for_file(to, &self.separator)?;
        }
        fs::rename(from, to)
    }
    async fn put(&self, path: &str, content: Vec<u8>) -> io::Result<()> {
        create_dir_all_for_file(path, &self.separator)?;
        fs::write(path, content)
    }
    async fn delete(&self, paths: &Vec<String>) -> io::Result<()> {
        for path in paths {
            fs::remove_dir_all(&path)?;
        }
        Ok(())
    }
    async fn files(&self, directory: &str, recursive: bool) -> io::Result<Vec<String>> {
        let mut result: Vec<String> = Vec::new();
        collect_files_from_dir_into_str_vec(&mut result, directory, recursive)?;
        Ok(result)
    }
    async fn directories(&self, directory: &str, recursive: bool) -> io::Result<Vec<String>> {
        let mut result: Vec<String> = Vec::new();
        collect_directories_from_dir_into_str_vec(&mut result, directory, recursive)?;
        Ok(result)
    }
    async fn make_directory(&self, path: &str) -> io::Result<()> {
        fs::create_dir_all(path)
    }
    async fn delete_directory(&self, directory: &str) -> io::Result<()> {
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
    async fn path(&self, url: &str) -> io::Result<String> {
        let url = url.trim();
        if !url.starts_with("http") {
            return Err(io::Error::other("Http protocol not found in the url."));
        }
        Ok(url.to_string())
    }
    async fn exists(&self, url: &str) -> io::Result<bool> {
        if let Ok(response) = reqwest::get(url).await {
            let status_code = response.status();

            if status_code.eq(&StatusCode::OK) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    async fn get(&self, url: &str) -> io::Result<Vec<u8>> {
        if let Ok(response) = reqwest::get(url).await {
            if let Ok(result) = response.bytes().await {
                return Ok(result.to_vec());
            }
        }
        Err(io::Error::new(
            ErrorKind::NotFound,
            ErrorKind::NotFound.to_string(),
        ))
    }
    // async fn read_stream(&self, url: &str) -> io::Result<impl futures_core::Stream<Item = reqwest::Result<Bytes>>> {
    //     if let Ok(response) = reqwest::get(url).await {
    //         return Ok(response.bytes_stream());
    //     }
    //     Err(io::Error::new(ErrorKind::NotFound, ErrorKind::NotFound.to_string()))
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fmt::format;
    use std::path::MAIN_SEPARATOR_STR;

    #[tokio::test]
    async fn test_local_disk_call_path() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_path
        let repository = DiskLocalRepository::new("/", "/");
        assert_eq!(&repository.root, "/");
        let repository = DiskLocalRepository::new("/app/", "/");
        assert_eq!(&repository.root, "/app");
        let repository = DiskLocalRepository::new("/app", "/");
        assert_eq!(&repository.root, "/app");
        assert_eq!(
            repository.path("/test").await.unwrap().as_str(),
            "/app/test"
        );
        assert_eq!(
            repository.path("/test ").await.unwrap().as_str(),
            "/app/test"
        );
        assert_eq!(
            repository.path(" /test ").await.unwrap().as_str(),
            "/app/test"
        );
        assert_eq!(
            repository.path(" test ").await.unwrap().as_str(),
            "/app/test"
        );
        assert_eq!(repository.path("test").await.unwrap().as_str(), "/app/test");

        let repository = DiskLocalRepository::new("C:\\", "\\");
        assert_eq!(&repository.root, "C:");
        let repository = DiskLocalRepository::new("C:\\app\\", "\\");
        assert_eq!(&repository.root, "C:\\app");
        let repository = DiskLocalRepository::new("C:\\app", "\\");
        assert_eq!(&repository.root, "C:\\app");
        assert_eq!(
            repository.path("\\test").await.unwrap().as_str(),
            "C:\\app\\test"
        );
        assert_eq!(
            repository.path("\\test ").await.unwrap().as_str(),
            "C:\\app\\test"
        );
        assert_eq!(
            repository.path(" \\test ").await.unwrap().as_str(),
            "C:\\app\\test"
        );
        assert_eq!(
            repository.path(" test ").await.unwrap().as_str(),
            "C:\\app\\test"
        );
        assert_eq!(
            repository.path("test").await.unwrap().as_str(),
            "C:\\app\\test"
        );
        assert_eq!(
            repository.path("/test/").await.unwrap().as_str(),
            "C:\\app\\/test/"
        );
    }

    #[tokio::test]
    async fn test_external_disk_call_path() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_external_disk_call_path
        let repository = DiskExternalRepository::new();
        let url = repository.path(".gitignore").await;
        assert!(url.is_err());
        let url = repository.path("http://localhost").await;
        assert!(url.is_ok());
        let url = repository.path("https://localhost").await;
        assert!(url.is_ok());
    }

    #[tokio::test]
    async fn test_local_disk_call_exists() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_exists
        let root = env::current_dir().unwrap();
        let repository = DiskLocalRepository::new(root.to_str().unwrap(), MAIN_SEPARATOR_STR);
        let exists_file_path = repository.path(".gitignore").await.unwrap();
        assert!(repository.exists(&exists_file_path).await.unwrap());
        let no_exists_file_path = repository.path(".git_ignore").await.unwrap();
        assert_eq!(
            repository.exists(&no_exists_file_path).await.unwrap(),
            false
        );
        let exists_dir_path = repository.path("src").await.unwrap();
        assert!(repository.exists(&exists_dir_path).await.unwrap());
        let no_exists_dir_path = repository.path("s_r_c").await.unwrap();
        assert_eq!(repository.exists(&no_exists_dir_path).await.unwrap(), false);
    }

    #[tokio::test]
    async fn test_external_disk_call_exists() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_external_disk_call_exists
        let repository = DiskExternalRepository::new();
        let url = repository.path(".gitignore").await;
        assert!(url.is_err());
        let url = repository.path("https://www.google.com/").await.unwrap();
        assert!(repository.exists(&url).await.unwrap());
        let url = repository
            .path("https://www.go_test_ogle.com/")
            .await
            .unwrap();
        assert_eq!(repository.exists(&url).await.unwrap(), false);
    }

    #[tokio::test]
    async fn test_local_disk_call_get() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_get
        let root = env::current_dir().unwrap();
        let repository = DiskLocalRepository::new(root.to_str().unwrap(), MAIN_SEPARATOR_STR);
        let path = repository.path(".gitignore").await.unwrap();
        let content = repository.get(&path).await.unwrap();
        let content_str = String::from_utf8(content).unwrap();

        assert_ne!(content_str.len(), 0);

        let path = repository.path(".git_ignore").await.unwrap();
        let error = repository.get(&path).await;

        assert!(error.is_err());
    }

    #[tokio::test]
    async fn test_external_disk_call_get() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_external_disk_call_get
        let repository = DiskExternalRepository::new();
        let path = repository
            .path("https://jsonplaceholder.typicode.com/todos/1")
            .await
            .unwrap();
        let content = repository.get(&path).await.unwrap();
        let content_str = String::from_utf8(content).unwrap();

        assert_ne!(content_str.len(), 0);

        let path = repository
            .path("https://jsonpla_test_ceholder.typi_test_code.com/todos/1")
            .await
            .unwrap();
        let error = repository.get(&path).await;

        assert!(error.is_err());
    }

    #[tokio::test]
    async fn test_local_disk_call_put() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_put
        let root = env::current_dir().unwrap();
        let repository = DiskLocalRepository::new(root.to_str().unwrap(), MAIN_SEPARATOR_STR);
        let dir_path = repository.path("/test_local_disk_call_put").await.unwrap();
        let path = repository
            .path("/test_local_disk_call_put/test.txt")
            .await
            .unwrap();
        let data = Vec::from(b"Test data");
        repository.put(&path, data).await.unwrap();
        assert!(fs::exists(&path).unwrap());
        fs::remove_dir_all(&dir_path).unwrap();
    }

    #[tokio::test]
    async fn test_local_disk_call_directories() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_directories
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, MAIN_SEPARATOR_STR);
        let test_dir = repository
            .path("/test_local_disk_call_directories")
            .await
            .unwrap();

        let mut paths: Vec<String> = Vec::new();
        paths.push(
            repository
                .path("/test_local_disk_call_directories/test1_1/test2_1")
                .await
                .unwrap(),
        );
        paths.push(
            repository
                .path("/test_local_disk_call_directories/test1_2/test2_2")
                .await
                .unwrap(),
        );
        paths.push(
            repository
                .path("/test_local_disk_call_directories/test1_3/test2_3/test1_1")
                .await
                .unwrap(),
        );
        paths.push(
            repository
                .path("/test_local_disk_call_directories/test1_3/test2_3/test1_1")
                .await
                .unwrap(),
        );

        for path in paths {
            fs::create_dir_all(&path).unwrap();
        }

        let mut directories = repository.directories(&test_dir, false).await.unwrap();
        directories.sort();
        assert_eq!(
            directories,
            Vec::from([
                format!("{}/test_local_disk_call_directories/test1_1", root),
                format!("{}/test_local_disk_call_directories/test1_2", root),
                format!("{}/test_local_disk_call_directories/test1_3", root),
            ])
        );
        let mut rec_directories = repository.directories(&test_dir, true).await.unwrap();
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
    #[tokio::test]
    async fn test_local_disk_call_files() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_files
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, MAIN_SEPARATOR_STR);
        let test_dir = repository
            .path("/test_local_disk_call_files")
            .await
            .unwrap();

        let mut paths: Vec<String> = Vec::new();
        paths.push(
            repository
                .path("/test_local_disk_call_files")
                .await
                .unwrap(),
        );
        paths.push(
            repository
                .path("/test_local_disk_call_files/test1_2/test2_2")
                .await
                .unwrap(),
        );

        for path in paths {
            fs::create_dir_all(&path).unwrap();
            let filename1 = format!("{}/{}", &path, "file1.txt");
            let filename2 = format!("{}/{}", &path, "file2.txt");
            fs::write(&filename1, "test").unwrap();
            fs::write(&filename2, "test").unwrap();
        }

        let mut files = repository.files(&test_dir, false).await.unwrap();
        files.sort();
        assert_eq!(
            files,
            Vec::from([
                format!("{}/test_local_disk_call_files/file1.txt", root),
                format!("{}/test_local_disk_call_files/file2.txt", root),
            ])
        );
        let mut rec_files = repository.files(&test_dir, true).await.unwrap();
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
    #[tokio::test]
    async fn test_local_disk_call_delete() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_delete
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, MAIN_SEPARATOR_STR);
        let test_dir1 = repository
            .path("/test_local_disk_call_delete1")
            .await
            .unwrap();
        let test_dir2 = repository
            .path("/test_local_disk_call_delete2")
            .await
            .unwrap();

        let mut del_paths: Vec<String> = Vec::new();
        del_paths.push(test_dir1.to_owned());
        del_paths.push(test_dir2.to_owned());

        let mut paths: Vec<String> = Vec::new();
        paths.push(
            repository
                .path("/test_local_disk_call_delete1/test1_1/test1_2")
                .await
                .unwrap(),
        );
        paths.push(
            repository
                .path("/test_local_disk_call_delete2/test1_2/test2_2")
                .await
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

        repository.delete(&del_paths).await.unwrap();

        assert!(!fs::exists(&test_dir1).unwrap());
        assert!(!fs::exists(&test_dir2).unwrap());
    }

    #[tokio::test]
    async fn test_local_disk_call_copy() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_copy
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, MAIN_SEPARATOR_STR);
        let dir_path = repository.path("/test_local_disk_call_copy").await.unwrap();
        let dir_path2 = repository
            .path("test_local_disk_call_copy/test")
            .await
            .unwrap();
        let file1_path = repository
            .path("/test_local_disk_call_copy/test/test1.txt")
            .await
            .unwrap();
        let file2_path = repository
            .path("/test_local_disk_call_copy/test2/test2.txt")
            .await
            .unwrap();
        fs::create_dir_all(dir_path2).unwrap();
        fs::write(&file1_path, "Test data").unwrap();
        repository.copy(&file1_path, &file2_path).await.unwrap();
        assert!(fs::exists(&file2_path).unwrap());
        fs::remove_dir_all(&dir_path).unwrap();
    }

    #[tokio::test]
    async fn test_local_disk_call_mv() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_mv
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, MAIN_SEPARATOR_STR);
        let dir_path = repository.path("/test_local_disk_call_mv").await.unwrap();
        let dir_path2 = repository
            .path("/test_local_disk_call_mv/test")
            .await
            .unwrap();
        let file1_path = repository
            .path("/test_local_disk_call_mv/test/test1.txt")
            .await
            .unwrap();
        let file2_path = repository
            .path("/test_local_disk_call_mv/test2/test2.txt")
            .await
            .unwrap();
        fs::create_dir_all(&dir_path2).unwrap();
        fs::write(&file1_path, "Test data").unwrap();
        repository.mv(&file1_path, &file2_path).await.unwrap();
        assert!(fs::exists(&file2_path).unwrap());
        assert!(!fs::exists(&file1_path).unwrap());
        fs::remove_dir_all(&dir_path).unwrap();
    }

    #[tokio::test]
    async fn test_local_disk_call_size() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_size
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, MAIN_SEPARATOR_STR);
        let dir_path = repository.path("/test_local_disk_call_size").await.unwrap();
        let file1_path = repository
            .path("/test_local_disk_call_size/test1.txt")
            .await
            .unwrap();
        fs::create_dir_all(&dir_path).unwrap();
        fs::write(&file1_path, "Test data").unwrap();
        let size = repository.size(&file1_path).await.unwrap();
        assert!(size > 0);
        fs::remove_dir_all(&dir_path).unwrap();
    }

    #[tokio::test]
    async fn test_local_disk_call_last_modified() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::repositories::disk::tests::test_local_disk_call_last_modified
        let root = env::current_dir().unwrap();
        let root = root.to_str().unwrap();
        let repository = DiskLocalRepository::new(root, MAIN_SEPARATOR_STR);
        let dir_path = repository
            .path("/test_local_disk_call_last_modified")
            .await
            .unwrap();
        let file1_path = repository
            .path("/test_local_disk_call_last_modified/test1.txt")
            .await
            .unwrap();
        fs::create_dir_all(&dir_path).unwrap();
        fs::write(&file1_path, "Test data").unwrap();
        let _ = repository.last_modified(&file1_path).await.unwrap();
        fs::remove_dir_all(&dir_path).unwrap();
    }
}
