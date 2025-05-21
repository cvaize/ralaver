use crate::connections::Connections;
use crate::diesel_mysql_connection::DieselMysqlPooledConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/");

pub fn migrate(c: &Connections) {
    let mut connection: DieselMysqlPooledConnection = c.mysql.get_ref().get().unwrap();
    let _ = connection.run_pending_migrations(MIGRATIONS);
}