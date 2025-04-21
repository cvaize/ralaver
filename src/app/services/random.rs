use rand::distr::uniform::{SampleRange, SampleUniform};
use rand::Rng;
use crate::helpers::get_sys_gettime_nsec;

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

    pub fn str_sys_gettime(&self, length: usize) -> String {
        let nsec = get_sys_gettime_nsec().to_string();
        let nsec_len = nsec.len();
        if length <= nsec_len {
            return nsec;
        }
        let length = length - nsec_len;

        let mut str = nsec;
        let str_ = self.str(length);
        str.push_str(&str_);
        str
    }
}

// #[derive(Debug, Clone, Copy)]
// pub enum RandomServiceError {
// }

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn str() {
        let str: String = RandomService::new().str(64);
        assert_eq!(64, str.len());
    }

    #[bench]
    fn bench_str(b: &mut Bencher) {
        b.iter(|| RandomService::new().str(64));
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

    #[bench]
    fn bench_range(b: &mut Bencher) {
        b.iter(|| RandomService::new().range(1..=100));
    }

    #[test]
    fn str_sys_gettime() {
        let random_service = RandomService::new();
        let str: String = random_service.str_sys_gettime(64);
        assert_eq!(64, str.len());
    }

    #[bench]
    fn bench_str_sys_gettime(b: &mut Bencher) {
        b.iter(|| RandomService::new().str_sys_gettime(64));
    }
}