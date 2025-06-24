#[path = "../config.rs"]
mod config;
#[path = "../errors.rs"]
mod errors;
#[path = "../app/connections/mysql.rs"]
mod connections_mysql;
#[path = "../migrations/mod.rs"]
mod migrations;

use config::Config;
use connections_mysql::{MysqlPool, MysqlPooledConnection};
use mysql::prelude::Queryable;
use mysql::{params, Row};
use std::collections::HashMap;
use std::env;

static MIGRATIONS_TABLE: &str = "__migrations";

fn main() {
    dotenv::dotenv().ok();
    let _ = env_logger::try_init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args: Vec<String> = env::args().collect();
    let command = args
        .get(1)
        .expect("The command is missing. Allowed commands: \"up\", \"down\".")
        .as_str();

    let config = Config::new();
    let mysql_pool: MysqlPool = connections_mysql::get_connection_pool(&config.db.mysql)
        .expect("Failed to create connection pool MysqlPool.");
    let mut mysql_connection: MysqlPooledConnection = mysql_pool
        .get()
        .expect("Failed to create connection MysqlPooledConnection.");

    match command {
        "up" => up(&config, &mut mysql_connection),
        "down" => down(&config, &mut mysql_connection),
        _ => panic!("Wrong command. Allowed commands: \"up\", \"down\"."),
    }
    log::info!("The migration was successful!");
}

fn create_migrations_table(mysql_connection: &mut MysqlPooledConnection) {
    let query = format!(
        "CREATE TABLE IF NOT EXISTS {MIGRATIONS_TABLE} (
   id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
   name VARCHAR(255) NOT NULL UNIQUE
);"
    );
    mysql_connection.query_drop(query).unwrap();
}

fn up(config: &Config, mysql_connection: &mut MysqlPooledConnection) -> () {
    create_migrations_table(mysql_connection);
    let migrations = load_migrations(mysql_connection);
    let mut index: HashMap<String, u64> = HashMap::new();
    for migration in migrations {
        index.insert(migration.name, migration.id);
    }

    let items = migrations::get_migrations();

    for (name, [up, _]) in items {
        if !index.contains_key(&name) {
            log::info!("Up migrating - {}", &name);
            up(config, mysql_connection);
            insert_migration(mysql_connection, &name);
            log::info!("Up migrated - {}", &name);
        }
    }

    ()
}

fn down(config: &Config, mysql_connection: &mut MysqlPooledConnection) -> () {
    create_migrations_table(mysql_connection);
    let migrations = load_migrations(mysql_connection);
    let migration = migrations.last();
    if migration.is_none() {
        return ();
    }
    let migration = migration.unwrap();
    let name = &migration.name;

    let items = migrations::get_migrations();
    let mut item: Option<[fn(&Config, &mut MysqlPooledConnection); 2]> = None;

    for (name_, item_) in items {
        if name_.eq(name) {
            item = Some(item_.clone());
        }
    }

    if let Some([_, down]) = item {
        log::info!("Down migrating - {}", name);
        down(config, mysql_connection);
        delete_migration(mysql_connection, name);
        log::info!("Down migrated - {}", name);
    } else {
        log::info!("Migration not found - {}", name);
    }

    ()
}

fn load_migrations(mysql_connection: &mut MysqlPooledConnection) -> Vec<Migration> {
    let query = format!("SELECT * FROM {MIGRATIONS_TABLE} ORDER BY id ASC");
    let rows = mysql_connection.query_iter(query).unwrap();

    let mut records: Vec<Migration> = Vec::new();
    for row in rows.into_iter() {
        if let Ok(row) = row {
            records.push(Migration::from_db_row(&row));
        }
    }

    records
}

fn insert_migration(mysql_connection: &mut MysqlPooledConnection, name: &str) {
    let query = format!("INSERT INTO {MIGRATIONS_TABLE} (name) VALUES (:name)");
    mysql_connection
        .exec_drop(query, params! {"name" => name})
        .unwrap();
}

fn delete_migration(mysql_connection: &mut MysqlPooledConnection, name: &str) {
    let query = format!("DELETE FROM {MIGRATIONS_TABLE} WHERE name=:name");
    mysql_connection
        .exec_drop(query, params! {"name" => name})
        .unwrap();
}

struct Migration {
    id: u64,
    name: String,
}

impl Migration {
    pub fn from_db_row(row: &Row) -> Self {
        Self {
            id: row.get("id").unwrap_or(0),
            name: row.get("name").unwrap_or("".to_string()),
        }
    }
}

