use mime::Mime;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
    Debug, Clone, Copy, Display, EnumString, Serialize, Deserialize, strum_macros::VariantNames, Eq, PartialEq,
)]
#[strum(serialize_all = "snake_case")]
pub enum Disk {
    Local,
}

#[derive(Debug, Default, Clone)]
pub struct UploadData {
    pub mime: Option<Mime>,
    pub filename: Option<String>,
    pub size: Option<u64>,
    pub is_public: Option<bool>,
    pub hash: Option<String>,
    pub user_id: Option<u64>,
}

impl Default for Disk {
    fn default() -> Disk {
        Disk::Local
    }
}