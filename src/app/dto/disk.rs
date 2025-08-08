use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    EnumString,
    Serialize,
    Deserialize,
    strum_macros::VariantNames,
    Eq,
    PartialEq,
)]
#[strum(serialize_all = "snake_case")]
pub enum Disk {
    Local,
}

impl Default for Disk {
    fn default() -> Disk {
        Disk::Local
    }
}
