use crate::app::connections::smtp::{get_smtp_transport, LettreSmtpTransport};
use crate::redis_connection::RedisPool;
use crate::services::BaseServices;
use crate::{diesel_mysql_connection, mysql_connection, redis_connection, DieselMysqlPool, MysqlPool};
use actix_web::web::Data;

pub fn smtp(s: &BaseServices) -> Data<LettreSmtpTransport> {
    let smtp: LettreSmtpTransport =
        get_smtp_transport(&s.config.get_ref().mail.smtp)
            .expect("Failed to create connection MysqlPool.");
    Data::new(smtp)
}
pub fn mysql(s: &BaseServices) -> Data<DieselMysqlPool> {
    let mysql_pool: DieselMysqlPool =
        diesel_mysql_connection::get_connection_pool(&s.config.get_ref().db.mysql)
            .expect("Failed to create connection MysqlPool.");
    Data::new(mysql_pool)
}
pub fn mysql2(s: &BaseServices) -> Data<MysqlPool> {
    let mysql_pool: MysqlPool =
        mysql_connection::get_connection_pool(&s.config.get_ref().db.mysql)
            .expect("Failed to create connection MysqlPool.");
    Data::new(mysql_pool)
}
pub fn redis(s: &BaseServices) -> Data<RedisPool> {
    let redis_pool: RedisPool =
        redis_connection::get_connection_pool(&s.config.get_ref().db.redis)
            .expect("Failed to create redis Pool.");
    Data::new(redis_pool)
}

pub struct Connections {
    pub smtp: Data<LettreSmtpTransport>,
    pub mysql: Data<DieselMysqlPool>,
    pub mysql2: Data<MysqlPool>,
    pub redis: Data<RedisPool>,
}

pub fn all(s: &BaseServices) -> Connections {
    let smtp: Data<LettreSmtpTransport> = smtp(s);
    let mysql: Data<DieselMysqlPool> = mysql(s);
    let mysql2: Data<MysqlPool> = mysql2(s);
    let redis: Data<RedisPool> = redis(s);

    Connections {
        smtp,
        mysql,
        mysql2,
        redis,
    }
}
