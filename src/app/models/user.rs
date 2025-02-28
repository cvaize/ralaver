use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = crate::schema::users)]
#[derive(Debug, Default, Serialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = crate::schema::users)]
#[derive(Debug, Default, Serialize)]
pub struct PublicUser {
    pub id: u64,
    pub username: String,
}

impl User {
    pub fn to_public_user(&self) -> PublicUser {
        PublicUser {
            id: self.id,
            username: self.username.clone(),
        }
    }
}
