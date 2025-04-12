#[derive(Debug, Clone)]
pub struct LogService {}

impl LogService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn info(&self, message: &str) {
        log::info!("{}", message);
    }

    pub fn warn(&self, message: &str) {
        log::warn!("{}", message);
    }

    pub fn error(&self, message: &str) {
        log::error!("{}", message);
    }
}
