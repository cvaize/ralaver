use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, VariantNames, EnumIter};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Role {
    pub id: u64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Display, EnumString, Serialize, Deserialize, VariantNames, EnumIter, Eq, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum RoleColumn {
    Id,
    Code,
    Name,
    Description,
    Permissions,
}