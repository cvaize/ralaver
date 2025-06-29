use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, VariantNames};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct File {
    pub id: u64,
    // The file name is made up of the hash, size, and extensions obtained when uploading the file, by mask: [hash]-[size].[extensions].
    pub filename: String,
    // The path where the file is saved on disk.
    pub path: String,
    // The file type.
    pub mime: Option<String>,
    // Hash of the sha256 file.
    pub hash: Option<String>,
    // The file size in bytes.
    pub size: Option<u64>,
    // The first user to upload the file.
    pub creator_user_id: Option<u64>,
    // The datetime of the file creation.
    pub created_at: Option<String>,
    // The datetime of the last file update.
    pub updated_at: Option<String>,
    // After this time, the file must be deleted.
    pub delete_at: Option<String>,
    // The datetime when the file was deleted.
    pub deleted_at: Option<String>,
    // Label: whether the file needs to be deleted.
    pub is_delete: bool,
    // Label: whether the file has been deleted.
    pub is_deleted: bool,
    // The disk where the file is stored.
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
    Path,
    Mime,
    Hash,
    Size,
    CreatorUserId,
    CreatedAt,
    UpdatedAt,
    DeleteAt,
    DeletedAt,
    IsDelete,
    IsDeleted,
    Disk,
}