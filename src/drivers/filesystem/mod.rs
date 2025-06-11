pub mod local;

use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
    Debug, Clone, Copy, Display, EnumString, Serialize, Deserialize, strum_macros::VariantNames,
)]
#[strum(serialize_all = "snake_case")]
pub enum FilesystemDriver {
    Local,
}
