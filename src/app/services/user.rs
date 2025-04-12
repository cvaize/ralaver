use crate::MysqlPool;
use actix_web::web::Data;
use strum_macros::{Display, EnumString};

pub struct UserService {
    db_pool: Data<MysqlPool>,
}

impl UserService {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        Self { db_pool }
    }

    pub fn create(&self) {}
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum UserServiceError {
    DbConnectionFail,
}

#[cfg(test)]
mod tests {
    #[test]
    fn create() {
        dbg!("test");
    }
}
