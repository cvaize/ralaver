use crate::{Config, MysqlPooledConnection};

pub mod users;
pub mod roles;

pub fn get_migrations() -> Vec<(String, [fn(&Config, &mut MysqlPooledConnection); 2])> {
    let mut items: Vec<(String, [fn(&Config, &mut MysqlPooledConnection); 2])> = Vec::new();

    items.push(("users".to_string(), [users::up, users::down]));
    items.push(("roles".to_string(), [roles::up, roles::down]));

    items
}