use r2d2_mysql::mysql::prelude::Queryable;
use crate::{Config, MysqlPooledConnection};

pub fn up(_: &Config, connection: &mut MysqlPooledConnection) {
    let query = "CREATE TABLE `roles` (
   `id` INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   `code` VARCHAR(255) NOT NULL UNIQUE,
   `name` VARCHAR(255) NOT NULL UNIQUE,
   `description` VARCHAR(255) NULL DEFAULT NULL,
   `permissions` JSON NULL DEFAULT NULL
);
";
    connection.query_drop(query).unwrap();

    let query = "
INSERT INTO `roles` (`id`, `code`, `name`, `permissions`) VALUES (1, 'admin', 'Admin', '[\"users_index\",
\"users_create\",
\"users_update\",
\"users_delete\",
\"roles_index\",
\"roles_create\",
\"roles_update\",
\"roles_delete\"]');
";
    connection.query_drop(query).unwrap();
}

pub fn down(_: &Config, connection: &mut MysqlPooledConnection){
    let query = "DROP TABLE `roles`;";
    connection.query_drop(query).unwrap();
}
