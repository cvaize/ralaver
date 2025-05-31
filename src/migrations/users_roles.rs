use crate::{Config, MysqlPooledConnection};
use r2d2_mysql::mysql::prelude::Queryable;

pub fn up(_: &Config, connection: &mut MysqlPooledConnection) {
    let query = "CREATE TABLE `users_roles` (
   `user_id` BIGINT UNSIGNED NOT NULL,
   `role_id` INT UNSIGNED NOT NULL
);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `users_roles` ADD UNIQUE `user_id_role_id` (`user_id`, `role_id`);";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `users_roles` ADD CONSTRAINT `users_roles_user_id_ref` FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE ON UPDATE CASCADE;";
    connection.query_drop(query).unwrap();

    let query = "ALTER TABLE `users_roles` ADD CONSTRAINT `users_roles_role_id_ref` FOREIGN KEY (`role_id`) REFERENCES `roles`(`id`) ON DELETE CASCADE ON UPDATE CASCADE;";
    connection.query_drop(query).unwrap();

    let query = "INSERT INTO `users_roles` (`user_id`, `role_id`) VALUES (1, 1);";
    connection.query_drop(query).unwrap();
}

pub fn down(_: &Config, connection: &mut MysqlPooledConnection) {
    let query = "DROP TABLE `users_roles`;";
    connection.query_drop(query).unwrap();
}
