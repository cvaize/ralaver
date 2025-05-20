use serde::{Serialize, Deserialize};
use crate::app::repositories::Value;
use r2d2_mysql::mysql::Row;

#[derive(Debug, Default, Serialize)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub password: Option<String>,
    pub locale: Option<String>,
    pub surname: Option<String>,
    pub name: Option<String>,
    pub patronymic: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct PrivateUserData {
    pub id: u64,
    pub email: String,
    pub password: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct NewUser {
    pub email: String,
    pub password: Option<String>,
    pub locale: Option<String>,
    pub surname: Option<String>,
    pub name: Option<String>,
    pub patronymic: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateUser {
    pub email: Option<Value<String>>,
    pub password: Option<Value<String>>,
    pub locale: Option<Value<String>>,
    pub surname: Option<Value<String>>,
    pub name: Option<Value<String>>,
    pub patronymic: Option<Value<String>>,
}

impl User {
    pub fn from_db_row(row: &Row) -> Self {
        Self {
            id: row.get("id").unwrap_or(0),
            email: row.get("email").unwrap_or("".to_string()),
            password: row.get("password").unwrap_or(None),
            locale: row.get("locale").unwrap_or(None),
            surname: row.get("surname").unwrap_or(None),
            name: row.get("name").unwrap_or(None),
            patronymic: row.get("patronymic").unwrap_or(None),
        }
    }

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

impl PrivateUserData {
    pub fn from_db_row(row: &Row) -> Self {
        Self {
            id: row.get("id").unwrap_or(0),
            email: row.get("email").unwrap_or("".to_string()),
            password: row.get("password").unwrap_or(None),
        }
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
