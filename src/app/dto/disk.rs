use mime::Mime;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
    Debug, Clone, Copy, Display, EnumString, Serialize, Deserialize, strum_macros::VariantNames, Eq, PartialEq,
)]
#[strum(serialize_all = "snake_case")]
pub enum Disk {
    Local,
    External,
}

#[derive(Debug, Default)]
pub struct UploadData {
    pub path: String,
    pub mime: Option<Mime>,
    pub filename: Option<String>,
    pub size: Option<u64>,
    pub is_public: Option<bool>,
    pub from_disk: Option<Disk>,
    pub to_disk: Option<Disk>,
    pub creator_user_id: Option<u64>,
}

impl Disk {
    pub fn default() ->  Self {
        Self::Local
    }
}