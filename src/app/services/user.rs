use crate::{MysqlPool, User};
use actix_web::web::Data;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use strum_macros::{Display, EnumString};

pub struct UserService {
    db_pool: Data<MysqlPool>,
}

impl UserService {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        Self { db_pool }
    }

    pub fn first_by_id(&self, user_id: u64) -> Result<User, UserServiceError> {
        let mut connection = self.db_pool.get_ref().get()
            .map_err(|e| {
                log::error!("UserService::first_by_id - {e}");
                UserServiceError::DbConnectionFail
            })?;

        let user = crate::schema::users::dsl::users
            .find(user_id)
            .select(User::as_select())
            .first(&mut connection)
            .map_err(|e| {
                log::error!("UserService::first_by_id - {e}");
                UserServiceError::Fail
            })?;
        Ok(user)
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum UserServiceError {
    DbConnectionFail,
    Fail,
}
