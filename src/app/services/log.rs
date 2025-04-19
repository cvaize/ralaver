#[derive(Debug, Clone)]
pub struct Log {}

impl Log {
    pub fn info(message: &str) {
        log::info!("{}", message);
    }

    pub fn warn(message: &str) {
        log::warn!("{}", message);
    }

    pub fn error(message: &str) {
        log::error!("{}", message);
    }
}
