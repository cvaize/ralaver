use crate::app::connections::smtp::{get_smtp_transport, LettreSmtpTransport};
use crate::redis_connection::RedisPool;
use crate::{mysql_connection, redis_connection, Config, MysqlPool};
use actix_web::web::Data;

pub fn smtp(config: &Config) -> Data<LettreSmtpTransport> {
    let smtp: LettreSmtpTransport =
        get_smtp_transport(&config.mail.smtp)
            .expect("Failed to create connection MysqlPool.");
    Data::new(smtp)
}
pub fn mysql(config: &Config) -> Data<MysqlPool> {
    let mysql_pool: MysqlPool =
        mysql_connection::get_connection_pool(&config.db.mysql)
            .expect("Failed to create connection MysqlPool.");
    Data::new(mysql_pool)
}
pub fn redis(config: &Config) -> Data<RedisPool> {
    let redis_pool: RedisPool =
        redis_connection::get_connection_pool(&config.db.redis)
            .expect("Failed to create redis Pool.");
    Data::new(redis_pool)
}

pub struct Connections {
    pub smtp: Data<LettreSmtpTransport>,
    pub mysql: Data<MysqlPool>,
    pub redis: Data<RedisPool>,
}

pub fn all(config: &Config) -> Connections {
    let smtp: Data<LettreSmtpTransport> = smtp(config);
    let mysql: Data<MysqlPool> = mysql(config);
    let redis: Data<RedisPool> = redis(config);

    Connections {
        smtp,
        mysql,
        redis,
    }
}
