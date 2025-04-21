pub mod errors;
pub mod auth;
pub mod home;
pub mod locale;
pub mod profile;
pub mod users;

use crate::model_redis_impl;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FormData<Fields> {
    pub form: Option<DefaultForm<Fields>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultForm<Fields> {
    pub fields: Option<Fields>,
    pub errors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultFields {
    pub email: Option<Field>,
    pub password: Option<Field>,
}

model_redis_impl!(DefaultFields);

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    pub value: Option<String>,
    pub errors: Option<Vec<String>>,
}

model_redis_impl!(Field);

impl<Fields> FormData<Fields> {
    pub fn empty() -> Self {
        Self { form: None }
    }
}

impl<Fields> DefaultForm<Fields> {
    pub fn empty() -> Self {
        Self {
            fields: None,
            errors: None,
        }
    }
}

// impl DefaultFields {
//     pub fn empty() -> Self {
//         Self {
//             email: None,
//             password: None,
//         }
//     }
// }

impl Field {
    pub fn empty() -> Self {
        Self {
            value: None,
            errors: None,
        }
    }
}
