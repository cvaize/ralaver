use crate::Config;
use crate::DbPool;
use actix_web::web::Data;
#[allow(unused_imports)]
use diesel::prelude::*;


pub struct UserService {
    config: Config,
    db_pool: Data<DbPool>,
}

#[derive(Debug, Clone, Copy)]
pub enum UserServiceError {
    DbConnectionFail,
}

impl UserService {
    pub fn new(
        config: Config,
        db_pool: Data<DbPool>,
    ) -> Self {
        Self {
            config,
            db_pool,
        }
    }


}
