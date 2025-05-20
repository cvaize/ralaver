use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Locale {
    pub code: String,
    pub short_name: String,
    pub full_name: String,
}