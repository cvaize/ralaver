use crate::app::connections::smtp::{get_smtp_transport, LettreSmtpTransport};
use crate::redis_connection::RedisPool;
use crate::services::BaseServices;
use crate::{mysql_connection, redis_connection, MysqlPool};
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::Key;
use actix_web::web::Data;

pub fn smtp(s: &BaseServices) -> Data<LettreSmtpTransport> {
    // Smtp
    let smtp: LettreSmtpTransport =
        get_smtp_transport(&s.config.get_ref().mail.smtp, s.log.get_ref())
            .expect("Failed to create connection MysqlPool.");
    Data::new(smtp)
}
pub fn mysql(s: &BaseServices) -> Data<MysqlPool> {
    let mysql_pool: MysqlPool =
        mysql_connection::get_connection_pool(&s.config.get_ref().db.mysql, s.log.get_ref())
            .expect("Failed to create connection MysqlPool.");
    Data::new(mysql_pool)
}
pub async fn session_redis(s: &BaseServices) -> (Key, RedisSessionStore) {
    let session_redis_secret: Key =
        redis_connection::get_session_secret(&s.config.db.redis);
    let session_redis_store: RedisSessionStore =
        redis_connection::get_session_store(&s.config.db.redis, s.log.get_ref())
            .await
            .expect("Failed to create session redis store.");

    (session_redis_secret, session_redis_store)
}
pub fn redis(s: &BaseServices) -> Data<RedisPool> {
    let redis_pool: RedisPool =
        redis_connection::get_connection_pool(&s.config.get_ref().db.redis, s.log.get_ref())
            .expect("Failed to create redis Pool.");
    Data::new(redis_pool)
}

pub struct Connections {
    pub session_redis_secret: Key,
    pub session_redis_store: RedisSessionStore,
    pub smtp: Data<LettreSmtpTransport>,
    pub mysql: Data<MysqlPool>,
    pub redis: Data<RedisPool>,
}

pub async fn all(s: &BaseServices) -> Connections {
    let smtp: Data<LettreSmtpTransport> = smtp(s);
    let mysql: Data<MysqlPool> = mysql(s);
    let (session_redis_secret, session_redis_store) = session_redis(s).await;
    let redis: Data<RedisPool> = redis(s);

    Connections {
        session_redis_secret,
        session_redis_store,
        smtp,
        mysql,
        redis,
    }
}
