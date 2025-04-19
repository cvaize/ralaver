use rand::distr::uniform::{SampleRange, SampleUniform};
use rand::Rng;

pub static CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

#[derive(Debug, Clone)]
pub struct RandomService {}

impl RandomService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn str(&self, length: usize) -> String {
        let mut rng = rand::rng();

        let str: String = (0..length)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        str
    }

    pub fn range<T: SampleUniform, R: SampleRange<T>>(&self, range: R) -> T {
        let mut rng = rand::rng();
        rng.random_range(range)
    }
}

// #[derive(Debug, Clone, Copy)]
// pub enum RandomServiceError {
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str() {
        let random_service = RandomService::new();
        let str: String = random_service.str(64);
        assert_eq!(64, str.len());
    }

    #[test]
    fn range() {
        let random_service = RandomService::new();
        let int: u32 = random_service.range(1..=1);
        assert_eq!(1, int);
        let int: u32 = random_service.range(2..=2);
        assert_eq!(2, int);
        let int: u32 = random_service.range(1..=50);
        assert_eq!(true, int >= 1 && int <= 50);
    }
}