mod disk;
mod file;
mod mysql;
mod redis;
mod role;
mod user;
mod user_file;

pub use self::disk::*;
pub use self::file::*;
pub use self::mysql::*;
pub use self::redis::*;
pub use self::role::*;
pub use self::user::*;
pub use self::user_file::*;
use serde::{Deserialize, Serialize};

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

#[derive(Debug)]
pub struct PaginateParams<Filter, Sort> {
    pub page: i64,
    pub per_page: i64,
    pub filters: Vec<Filter>,
    pub sorts: Vec<Sort>,
}

impl<Filter, Sort> PaginateParams<Filter, Sort> {
    pub fn new(page: i64, per_page: i64, filters: Vec<Filter>, sorts: Vec<Sort>) -> Self {
        Self {
            page,
            per_page,
            filters,
            sorts,
        }
    }

    pub fn simple(page: i64, per_page: i64) -> Self {
        Self {
            page,
            per_page,
            filters: Vec::new(),
            sorts: Vec::new(),
        }
    }

    pub fn one() -> Self {
        Self {
            page: 1,
            per_page: 1,
            filters: Vec::new(),
            sorts: Vec::new(),
        }
    }
}
