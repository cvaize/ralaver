use serde_derive::{Deserialize, Serialize};
use crate::Permission;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Role {
    pub id: u64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Option<Vec<Permission>>,
}