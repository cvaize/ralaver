use crate::MysqlPool;
use actix_web::web::Data;
use strum_macros::{Display, EnumString};

#[allow(dead_code)]
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

// #[cfg(test)]
// mod tests {
//     use crate::Config;
//
//     #[test]
//     fn create() {
//         let config = Config::new();
//
//         dbg!(&config.db.mysql.url);
//         dbg!("test");
//     }
// }
