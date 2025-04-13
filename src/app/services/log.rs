#[derive(Debug, Clone)]
pub struct LogService {}

impl LogService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn info(&self, message: &str) {
        Self::print_info(message);
    }

    pub fn warn(&self, message: &str) {
        Self::print_warn(message);
    }

    pub fn error(&self, message: &str) {
        Self::print_error(message);
    }

    pub fn print_info(message: &str) {
        log::info!("{}", message);
    }

    pub fn print_warn(message: &str) {
        log::warn!("{}", message);
    }

    pub fn print_error(message: &str) {
        log::error!("{}", message);
    }
}
