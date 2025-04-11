use crate::Config;

#[derive(Debug, Clone)]
pub struct LogService {
    config: Config,
}

impl LogService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn info(&self, message: &str) {
        log::info!(message);
    }

    pub fn warn(&self, message: &str) {
        log::warn!(message);
    }

    pub fn error(&self, message: &str) {
        log::error!(message);
    }
}
