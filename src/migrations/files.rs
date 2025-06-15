use r2d2_mysql::mysql::prelude::Queryable;
use crate::{Config, MysqlPooledConnection};

pub fn up(_: &Config, connection: &mut MysqlPooledConnection) {
    let query = "CREATE TABLE `files` (
   `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   `name` VARCHAR(255) NOT NULL,
   `public_path` VARCHAR(2048) CHARACTER SET ascii COLLATE ascii_bin NULL DEFAULT NULL,
   `local_path` VARCHAR(2048) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
   `mime` VARCHAR(255) CHARACTER SET ascii COLLATE ascii_bin NULL DEFAULT NULL,
   `hash` VARCHAR(128) CHARACTER SET ascii COLLATE ascii_bin NULL DEFAULT NULL,
   `size` BIGINT UNSIGNED NULL DEFAULT NULL,
   `creator_user_id` BIGINT UNSIGNED NULL DEFAULT NULL,
   `created_at` DATETIME NULL DEFAULT NULL,
   `updated_at` DATETIME NULL DEFAULT NULL,
   `file_delete_at` DATETIME NULL DEFAULT NULL,
   `file_deleted_at` DATETIME NULL DEFAULT NULL,
   `deleted_at` DATETIME NULL DEFAULT NULL,
   `is_public` BOOLEAN NOT NULL DEFAULT FALSE,
   `disk` VARCHAR(255) CHARACTER SET ascii COLLATE ascii_bin NOT NULL
);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `public_path` (`public_path`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `local_path` (`local_path`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `is_public` (`is_public`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `disk` (`disk`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `mime` (`mime`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `hash` (`hash`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `creator_user_id` (`creator_user_id`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `deleted_at` (`deleted_at`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `file_delete_at` (`file_delete_at`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `file_deleted_at` (`file_deleted_at`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `file_deleted` (`file_delete_at`, `file_deleted_at`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD UNIQUE `disk_local_path` (`disk`, `local_path`);
";
    connection.query_drop(query).unwrap();
}

pub fn down(_: &Config, connection: &mut MysqlPooledConnection){
    let query = "DROP TABLE `files`;";
    connection.query_drop(query).unwrap();
}