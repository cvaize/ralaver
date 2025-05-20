mod user;

pub use self::user::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Value<T> {
    Set(T),
    #[default]
    Null,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginationResult<U> {
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
    pub total_records: i64,
    pub records: Vec<U>,
}

impl<U> PaginationResult<U> {
    pub fn new(page: i64, per_page: i64, total_records: i64, records: Vec<U>) -> Self {
        Self {
            page,
            per_page,
            total_pages: (total_records as f64 / per_page as f64).ceil() as i64,
            total_records,
            records,
        }
    }
}
