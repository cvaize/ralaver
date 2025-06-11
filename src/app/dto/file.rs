use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, VariantNames, EnumIter};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct File {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Copy, Display, EnumString, Serialize, Deserialize, VariantNames, EnumIter, Eq, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum FileColumn {
    Id,
    Name,
    Url,
    IsDeleted,
}