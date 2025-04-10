use crate::Config;
use actix_web::web::Data;

pub struct LogService {
    config: Config,
}

impl LogService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn success(&self, message: String) {
        println!("SUCCESS: {}", message);
    }

    pub fn info(&self, message: String) {
        println!("INFO: {}", message);
    }

    pub fn warning(&self, message: String) {
        println!("WARNING: {}", message);
    }

    pub fn error(&self, message: String) {
        eprintln!("ERROR: {}", message);
    }
}
