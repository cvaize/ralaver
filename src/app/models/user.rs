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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patronymic: Option<String>,
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

impl User {
    pub fn get_full_name_with_id_and_email(&self) -> String {
        let mut full_name = "".to_string();

        if let Some(surname) = &self.surname {
            full_name.push_str(surname);
        }

        if let Some(name) = &self.name {
            if full_name.len() != 0 {
                full_name.push(' ');
            }
            full_name.push_str(name);
        }

        if let Some(patronymic) = &self.patronymic {
            if full_name.len() != 0 {
                full_name.push(' ');
            }
            full_name.push_str(patronymic);
        }

        if full_name.len() != 0 {
            full_name.push_str(" - ");
        }

        full_name.push_str(&self.email);

        full_name.push_str("(ID:");
        full_name.push_str(&self.id.to_string());
        full_name.push(')');

        full_name
    }
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
