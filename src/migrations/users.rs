use r2d2_mysql::mysql::prelude::Queryable;
use crate::{Config, MysqlPooledConnection};

pub fn up(_: &Config, connection: &mut MysqlPooledConnection) {
    let query = "CREATE TABLE `users` (
   `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   `email` VARCHAR(255) NOT NULL UNIQUE,
   `password` VARCHAR(255) NULL DEFAULT NULL,
   `locale` VARCHAR(6) NULL DEFAULT NULL,
   `surname` VARCHAR(255) NULL DEFAULT NULL,
   `name` VARCHAR(255) NULL DEFAULT NULL,
   `patronymic` VARCHAR(255) NULL DEFAULT NULL,
   `is_super_admin` BOOLEAN NOT NULL DEFAULT FALSE
);
";
    connection.query_drop(query).unwrap();
    let query = "
INSERT INTO `users` (`id`, `email`, `is_super_admin`) VALUES (1, 'admin@admin.example', true);
";
    connection.query_drop(query).unwrap();
}

pub fn down(_: &Config, connection: &mut MysqlPooledConnection){
    let query = "DROP TABLE `users`;";
    connection.query_drop(query).unwrap();
}