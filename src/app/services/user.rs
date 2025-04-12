use crate::MysqlPool;
use crate::{Config, LogService};
use actix_web::web::Data;
#[allow(unused_imports)]
use diesel::prelude::*;
use strum_macros::{Display, EnumString};

pub struct UserService {
    config: Data<Config>,
    db_pool: Data<MysqlPool>,
    log_service: Data<LogService>,
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum UserServiceError {
    DbConnectionFail,
}

impl UserService {
    pub fn new(
        config: Data<Config>,
        db_pool: Data<MysqlPool>,
        log_service: Data<LogService>,
    ) -> Self {
        Self {
            config,
            db_pool,
            log_service,
        }
    }
}
