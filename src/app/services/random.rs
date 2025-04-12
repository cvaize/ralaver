use actix_web::web::Data;
use rand::Rng;

pub static CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

pub struct RandomService {}

impl RandomService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn str(&self, length: usize) -> String {
        let mut rng = rand::thread_rng();

        let str: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        str
    }

    pub fn str2(&self, length: usize) -> String {
        let mut rng = rand::thread_rng();

        let str: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        str
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
        let random_service = Data::new(RandomService::new());
        let str: String = random_service.str2(10);

        dbg!(&str);

        assert_eq!(10, str.len());
    }
}