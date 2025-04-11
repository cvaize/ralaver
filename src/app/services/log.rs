use crate::Config;

#[derive(Debug, Clone)]
pub struct LogService {
    config: Config,
}

impl LogService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn success(&self, message: &str) {
        println!("SUCCESS: {}", message);
    }

    pub fn info(&self, message: &str) {
        println!("INFO: {}", message);
    }

    pub fn warning(&self, message: &str) {
        println!("WARNING: {}", message);
    }

    pub fn error(&self, message: &str) {
        eprintln!("ERROR: {}", message);
    }
}
