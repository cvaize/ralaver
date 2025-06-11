use r2d2_mysql::mysql::prelude::Queryable;
use crate::{Config, MysqlPooledConnection};

pub fn up(_: &Config, connection: &mut MysqlPooledConnection) {
    let query = "CREATE TABLE `files` (
   `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   `url` VARCHAR(2048) CHARACTER SET latin1 COLLATE latin1_bin NOT NULL,
   `name` VARCHAR(255) NOT NULL,
   `public_path` VARCHAR(255) NULL DEFAULT NULL,
   `local_path` VARCHAR(255) NULL DEFAULT NULL,
   `mime` VARCHAR(255) NULL DEFAULT NULL,
   `hash` VARCHAR(64) NULL DEFAULT NULL,
   `size` BIGINT UNSIGNED NULL DEFAULT NULL,
   `creator_user_id` BIGINT UNSIGNED NULL DEFAULT NULL,
   `created_at` DATETIME NULL DEFAULT NULL,
   `updated_at` DATETIME NULL DEFAULT NULL,
   `file_delete_at` DATETIME NULL DEFAULT NULL,
   `file_deleted_at` DATETIME NULL DEFAULT NULL,
   `deleted_at` DATETIME NULL DEFAULT NULL,
   `is_public` BOOLEAN NOT NULL DEFAULT FALSE,
   `is_deleted` BOOLEAN NOT NULL DEFAULT FALSE,
   `disk` VARCHAR(255) NULL DEFAULT NULL
);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `url` (`url`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `is_public` (`is_public`);
";
    connection.query_drop(query).unwrap();

    let query = "
ALTER TABLE `files` ADD INDEX `is_deleted` (`is_deleted`);
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
}

pub fn down(_: &Config, connection: &mut MysqlPooledConnection){
    let query = "DROP TABLE `files`;";
    connection.query_drop(query).unwrap();
}