use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, VariantNames};

// Files belonging to users.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserFile {
    pub id: u64,
    // The user who uploaded the file.
    pub user_id: u64,
    // Relation to the files table.
    pub file_id: u64,
    // The file name.
    pub filename: Option<String>,
    // The path or url where you can get the file.
    pub path: Option<String>,
    // The filename received during the upload.
    pub upload_filename: Option<String>,
    // The file type received during the upload.
    pub mime: Option<String>,
    // The datetime of the file creation.
    pub created_at: Option<String>,
    // The datetime of the last file update.
    pub updated_at: Option<String>,
    // The datetime when the file was deleted.
    pub deleted_at: Option<String>,
    // Label: whether the file has been deleted.
    pub is_deleted: bool,
    // Label: public file or not.
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
pub enum UserFileColumn {
    Id,
    UserId,
    FileId,
    Filename,
    Path,
    UploadFilename,
    Mime,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    IsDeleted,
    IsPublic,
    Disk,
}
