use r2d2_mysql::mysql::prelude::Queryable;
use crate::{Config, MysqlPooledConnection};

pub fn up(_: &Config, connection: &mut MysqlPooledConnection) {
    let query = "CREATE TABLE `roles_permissions` (
   `role_id` INT UNSIGNED NOT NULL,
   `permission_code` VARCHAR(255) NOT NULL
);
";
    connection.query_drop(query).unwrap();
    let query = "
ALTER TABLE `roles_permissions` ADD UNIQUE `role_id_permission_code` (`role_id`, `permission_code`);
";
    connection.query_drop(query).unwrap();
    let query = "
ALTER TABLE `roles_permissions` ADD CONSTRAINT `roles_permissions_role_id_ref` FOREIGN KEY (`role_id`) REFERENCES `roles`(`id`) ON DELETE CASCADE ON UPDATE CASCADE;
";
    connection.query_drop(query).unwrap();
    let query = "
INSERT INTO `roles_permissions` (`role_id`, `permission_code`) VALUES (1, 'users_index'),
(1, 'users_create'),(1, 'users_update'),(1, 'users_delete'),(1, 'roles_index'),(1, 'roles_create'),
(1, 'roles_update'),(1, 'roles_delete');
";
    connection.query_drop(query).unwrap();
}

pub fn down(_: &Config, connection: &mut MysqlPooledConnection){
    let query = "DROP TABLE `roles_permissions`;";
    connection.query_drop(query).unwrap();
}