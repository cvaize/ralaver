use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub struct AppError(pub Option<String>);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(ref err) = self.0 {
            write!(f, "{}", err)?;
        }
        Ok(())
    }
}