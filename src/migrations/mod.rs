use crate::{Config, MysqlPooledConnection};

pub mod users;
pub mod roles;
pub mod users_roles;
pub mod roles_permissions;

pub fn get_migrations() -> Vec<(String, [fn(&Config, &mut MysqlPooledConnection); 2])> {
    let mut items: Vec<(String, [fn(&Config, &mut MysqlPooledConnection); 2])> = Vec::new();

    items.push(("users".to_string(), [users::up, users::down]));
    items.push(("roles".to_string(), [roles::up, roles::down]));
    items.push(("users_roles".to_string(), [users_roles::up, users_roles::down]));
    items.push(("roles_permissions".to_string(), [roles_permissions::up, roles_permissions::down]));

    items
}