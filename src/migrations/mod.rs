use std::collections::HashMap;
use crate::{Config, MysqlPooledConnection};

pub mod users;

pub fn get_migrations() -> HashMap<String, [fn(&Config, &mut MysqlPooledConnection); 2]> {
    let mut items: HashMap<String, [fn(&Config, &mut MysqlPooledConnection); 2]> = HashMap::new();

    items.insert("users".to_string(), [users::up, users::down]);

    items
}