use r2d2_mysql::mysql::prelude::Queryable;
use crate::{Config, MysqlPooledConnection};

pub fn up(_: &Config, connection: &mut MysqlPooledConnection) {
    let query = "CREATE TABLE `files` (
   `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   `name` VARCHAR(255) NOT NULL,
   `url` VARCHAR(2048) CHARACTER SET latin1 COLLATE latin1_bin NOT NULL UNIQUE,
   `is_deleted` BOOLEAN NOT NULL DEFAULT FALSE
);
";
    connection.query_drop(query).unwrap();
}

pub fn down(_: &Config, connection: &mut MysqlPooledConnection){
    let query = "DROP TABLE `files`;";
    connection.query_drop(query).unwrap();
}