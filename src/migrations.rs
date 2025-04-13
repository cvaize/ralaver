use crate::connections::Connections;
use crate::mysql_connection::MysqlPooledConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/");

pub fn migrate(c: &Connections) {
    let mut connection: MysqlPooledConnection = c.mysql.get_ref().get().unwrap();
    let _ = connection.run_pending_migrations(MIGRATIONS);
}