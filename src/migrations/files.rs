use crate::{Config, MysqlPooledConnection};
use mysql::prelude::Queryable;

fn create_users_files_table(connection: &mut MysqlPooledConnection) {
    let query = "CREATE TABLE `users_files` (
   `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   `file_id` BIGINT UNSIGNED NOT NULL COMMENT 'Relation to the files table.',
   `path` VARCHAR(2048) CHARACTER SET ascii COLLATE ascii_bin NOT NULL COMMENT 'The path or url where you can get the file.',
   `mime` VARCHAR(255) CHARACTER SET ascii COLLATE ascii_bin NULL DEFAULT NULL COMMENT 'The file type received during the upload.',
   `upload_filename` VARCHAR(255) NULL DEFAULT NULL COMMENT 'The filename received during the upload.',
   `filename` VARCHAR(255) CHARACTER SET ascii COLLATE ascii_bin NOT NULL COMMENT 'The file name.',
   `user_id` BIGINT UNSIGNED NOT NULL COMMENT 'The user who uploaded the file.',
   `created_at` DATETIME NULL DEFAULT NULL COMMENT 'The datetime of the file creation.',
   `updated_at` DATETIME NULL DEFAULT NULL COMMENT 'The datetime of the last file update.',
   `deleted_at` DATETIME NULL DEFAULT NULL COMMENT 'The datetime when the file was deleted.',
   `is_deleted` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Label: whether the file has been deleted.',
   `is_public` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Label: public file or not.'
) COMMENT 'Files belonging to users.';";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `users_files` ADD UNIQUE `file_user_udx` (`user_id`, `file_id`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `users_files` ADD INDEX `user_idx` (`user_id`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `users_files` ADD INDEX `file_idx` (`file_id`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `users_files` ADD INDEX `path_idx` (`path`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `users_files` ADD INDEX `filename_idx` (`filename`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `users_files` ADD INDEX `upload_filename_idx` (`upload_filename`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `users_files` ADD INDEX `is_public_idx` (`is_public`);";
    connection.query_drop(query).unwrap();
}

fn create_files_table(connection: &mut MysqlPooledConnection) {
    let query = "CREATE TABLE `files` (
   `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   `filename` VARCHAR(2048) CHARACTER SET ascii COLLATE ascii_bin NOT NULL COMMENT 'The file name is made up of the hash, size, and extensions obtained when uploading the file, by mask: [hash]-[size].[extensions].',
   `path` VARCHAR(2048) CHARACTER SET ascii COLLATE ascii_bin NOT NULL COMMENT 'The path where the file is saved on disk.',
   `mime` VARCHAR(255) CHARACTER SET ascii COLLATE ascii_bin NULL DEFAULT NULL COMMENT 'The file type.',
   `hash` VARCHAR(64) CHARACTER SET ascii COLLATE ascii_bin NULL DEFAULT NULL COMMENT 'Hash of the sha256 file.',
   `size` BIGINT UNSIGNED NULL DEFAULT NULL COMMENT 'The file size in bytes.',
   `creator_user_id` BIGINT UNSIGNED NULL DEFAULT NULL COMMENT 'The first user to upload the file.',
   `created_at` DATETIME NULL DEFAULT NULL COMMENT 'The datetime of the file creation.',
   `updated_at` DATETIME NULL DEFAULT NULL COMMENT 'The datetime of the last file update.',
   `delete_at` DATETIME NULL DEFAULT NULL COMMENT 'After this time, the file must be deleted.',
   `deleted_at` DATETIME NULL DEFAULT NULL COMMENT 'The datetime when the file was deleted.',
   `is_delete` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Label: whether the file needs to be deleted.',
   `is_deleted` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Label: whether the file has been deleted.',
   `disk` VARCHAR(255) CHARACTER SET ascii COLLATE ascii_bin NOT NULL COMMENT 'The disk where the file is stored.'
) COMMENT 'The file table.';";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `files` ADD UNIQUE `disk_path_udx` (`disk`, `path`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `files` ADD INDEX `creator_user_idx` (`creator_user_id`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `files` ADD INDEX `disk_idx` (`disk`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `files` ADD INDEX `path_idx` (`path`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `files` ADD INDEX `filename_idx` (`filename`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `files` ADD INDEX `is_delete_idx` (`is_delete`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `files` ADD INDEX `is_deleted_idx` (`is_deleted`);";
    connection.query_drop(query).unwrap();
}

pub fn up(_: &Config, connection: &mut MysqlPooledConnection) {
    create_users_files_table(connection);
    create_files_table(connection);
}

pub fn down(_: &Config, connection: &mut MysqlPooledConnection) {
    connection.query_drop("DROP TABLE `users_files`;").unwrap();
    connection.query_drop("DROP TABLE `files`;").unwrap();
}
