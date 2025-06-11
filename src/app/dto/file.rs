use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, VariantNames};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct File {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub public_path: Option<String>,
    pub local_path: Option<String>,
    pub mime: Option<String>,
    pub hash: Option<String>,
    pub size: Option<u64>,
    pub creator_user_id: Option<u64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub file_delete_at: Option<String>,
    pub file_deleted_at: Option<String>,
    pub deleted_at: Option<String>,
    pub disk: Option<String>,
    pub is_public: bool,
    pub is_deleted: bool,
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
    Name,
    Url,
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
    Disk,
    IsPublic,
    IsDeleted,
}
