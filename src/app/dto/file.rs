use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;
use strum_macros::{Display, EnumIter, EnumString, VariantNames};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct File {
    pub id: u64,
    pub filename: Option<String>,
    pub public_path: Option<String>,
    pub local_path: String,
    pub mime: Option<String>,
    pub hash: Option<String>,
    pub size: Option<u64>,
    pub creator_user_id: Option<u64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub file_delete_at: Option<String>,
    pub file_deleted_at: Option<String>,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub is_public: bool,
    pub disk: String,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    EnumString,
    Serialize,
    Deserialize,
    VariantNames,
    EnumIter,
    Eq,
    PartialEq,
)]
#[strum(serialize_all = "snake_case")]
pub enum FileColumn {
    Id,
    Filename,
    PublicPath,
    LocalPath,
    Mime,
    Hash,
    Size,
    CreatorUserId,
    CreatedAt,
    UpdatedAt,
    FileDeleteAt,
    FileDeletedAt,
    DeletedAt,
    IsPublic,
    IsDeleted,
    Disk,
}
