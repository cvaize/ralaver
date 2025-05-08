use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Selectable, Debug, Default, Serialize)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: u64,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Default, Serialize)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = crate::schema::users)]
pub struct PrivateUserData {
    pub id: u64,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, Debug, Default, Serialize)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patronymic: Option<String>,
}

impl NewUser {
    pub fn empty(email: String) -> Self {
        Self {
            email,
            password: None,
            locale: None,
            surname: None,
            name: None,
            patronymic: None,
        }
    }
}
