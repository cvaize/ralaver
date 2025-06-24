use crate::{Config, MysqlPooledConnection};
use mysql::prelude::Queryable;

fn create_users_files_table(connection: &mut MysqlPooledConnection) {
    let query = "CREATE TABLE `users_files` (
   `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   `file_id` BIGINT UNSIGNED NOT NULL COMMENT 'Связь с таблицей файлов.',
   `path` VARCHAR(2048) CHARACTER SET ascii COLLATE ascii_bin NOT NULL COMMENT 'Путь или url, по которым можно получить файл.',
   `mime` VARCHAR(255) CHARACTER SET ascii COLLATE ascii_bin NULL DEFAULT NULL COMMENT 'Тип файла полученный при загрузке.',
   `filename` VARCHAR(255) NOT NULL COMMENT 'Название файла полученное при загрузке.',
   `user_id` BIGINT UNSIGNED NOT NULL COMMENT 'Пользователь загрузивший файл.',
   `created_at` DATETIME NULL DEFAULT NULL COMMENT 'Время создания файла.',
   `updated_at` DATETIME NULL DEFAULT NULL COMMENT 'Время последнего обновления файла.',
   `deleted_at` DATETIME NULL DEFAULT NULL COMMENT 'Время удаления файла.',
   `is_deleted` BOOLEAN NOT NULL DEFAULT FALSE 'Метка: удалён ли файл.',
) COMMENT 'Таблица принадлежности файлов пользователям.';";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `users_files` ADD UNIQUE `file_user_udx` (`user_id`, `file_id`);";
    connection.query_drop(query).unwrap();
}

fn create_files_table(connection: &mut MysqlPooledConnection) {
    let query = "CREATE TABLE `files` (
   `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   `filename` VARCHAR(2048) CHARACTER SET ascii COLLATE ascii_bin NOT NULL COMMENT 'Название файла составленное из хеша, размера и расширений полученных при загрузке файла, по маске: [hash]-[size].[extensions].',
   `path` VARCHAR(2048) CHARACTER SET ascii COLLATE ascii_bin NOT NULL COMMENT 'Путь по которому сохранён файл на диске.',
   `mime` VARCHAR(255) CHARACTER SET ascii COLLATE ascii_bin NULL DEFAULT NULL COMMENT 'Тип файла.',
   `hash` VARCHAR(64) CHARACTER SET ascii COLLATE ascii_bin NULL DEFAULT NULL COMMENT 'Hash sha256 файла.',
   `size` BIGINT UNSIGNED NULL DEFAULT NULL COMMENT 'Размер файла в байтах.',
   `creator_user_id` BIGINT UNSIGNED NULL DEFAULT NULL COMMENT 'Первый загрузивший файл пользователь.',
   `created_at` DATETIME NULL DEFAULT NULL COMMENT 'Время создания файла.',
   `updated_at` DATETIME NULL DEFAULT NULL COMMENT 'Время последнего обновления файла.',
   `delete_at` DATETIME NULL DEFAULT NULL COMMENT 'По прошествию этого времени файл нужно удалить.',
   `deleted_at` DATETIME NULL DEFAULT NULL COMMENT 'Время удаления файла.',
   `is_delete` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Метка: нужно ли удалить файл.',
   `is_deleted` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Метка: удалён ли файл.',
   `disk` VARCHAR(255) CHARACTER SET ascii COLLATE ascii_bin NOT NULL COMMENT 'Диск на котором хранится файл.'
) COMMENT 'Таблица файлов.';";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `files` ADD UNIQUE `disk_path_udx` (`disk`, `path`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `files` ADD INDEX `creator_user_idx` (`creator_user_id`);";
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
