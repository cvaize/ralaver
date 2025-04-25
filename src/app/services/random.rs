use rand::distr::uniform::{SampleRange, SampleUniform};
use rand::distr::{Alphanumeric, SampleString};
use rand::Rng;
use crate::helpers::get_sys_gettime_nsec;

#[derive(Debug, Clone)]
pub struct RandomService {}

impl RandomService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn str(&self, length: usize) -> String {
        let mut rng = rand::rng();
        Alphanumeric.sample_string(&mut rng, length)
    }

    pub fn range<T: SampleUniform, R: SampleRange<T>>(&self, range: R) -> T {
        let mut rng = rand::rng();
        rng.random_range(range)
    }

    pub fn bytes_16(&self) -> [u8; 16] {
        // TODO: Оптимизировать. Скорее всего стоит инициализировать rand::rng() и использовать повторно
        let mut rng = rand::rng();
        rng.random()
    }

    pub fn bytes_32(&self) -> [u8; 32] {
        // TODO: Оптимизировать. Скорее всего стоит инициализировать rand::rng() и использовать повторно
        let mut rng = rand::rng();
        rng.random()
    }

    pub fn bytes_64(&self) -> [u8; 64] {
        // TODO: Оптимизировать. Скорее всего стоит инициализировать rand::rng() и использовать повторно
        let mut rng = rand::rng();
        rng.random()
    }

    pub fn bytes_128(&self) -> [u8; 128] {
        // TODO: Оптимизировать. Скорее всего стоит инициализировать rand::rng() и использовать повторно
        let mut rng = rand::rng();
        rng.random()
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

    pub fn str_sys_gettime2(&self, length: usize) -> (String, String) {
        let nsec = get_sys_gettime_nsec().to_string();
        let nsec_len = nsec.len();
        if length <= nsec_len {
            return (nsec.to_owned(), nsec);
        }
        let length = length - nsec_len;

        let mut str1 = nsec.to_owned();
        let mut str2 = nsec;
        let str1_ = self.str(length);
        let str2_ = self.str(length);
        str1.push_str(&str1_);
        str2.push_str(&str2_);
        (str1, str2)
    }
}

// #[derive(Debug, Clone, Copy)]
// pub enum RandomServiceError {
// }

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use actix_web::web::Data;

    static CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    #[test]
    fn random() {
        let bytes: [u8; 128] = RandomService::new().bytes_128();
        assert_eq!(128, bytes.len());
    }

    #[test]
    fn str() {
        let str: String = RandomService::new().str(64);
        assert_eq!(64, str.len());
    }

    #[bench]
    fn bench_str(b: &mut Bencher) {
        let random_service = Data::new(RandomService::new());
        b.iter(|| random_service.get_ref().str(64));
    }

    #[bench]
    fn bench_lib_str(b: &mut Bencher) {
        let length = 64;
        b.iter(|| {
            let mut rng = rand::rng();
            let _ = Alphanumeric.sample_string(&mut rng, length);
        });
    }

    #[bench]
    fn bench_custom_str(b: &mut Bencher) {
        let length = 64;
        b.iter(|| {
            let mut rng = rand::rng();

            let _: String = (0..length)
                .map(|_| {
                    let idx = rng.random_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();
        });
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
        let random_service = Data::new(RandomService::new());
        b.iter(|| random_service.get_ref().range(1..=100));
    }

    #[test]
    fn str_sys_gettime() {
        let random_service = RandomService::new();
        let str: String = random_service.str_sys_gettime(64);
        assert_eq!(64, str.len());
    }

    #[bench]
    fn bench_str_sys_gettime(b: &mut Bencher) {
        let random_service = Data::new(RandomService::new());
        b.iter(|| random_service.get_ref().str_sys_gettime(64));
    }

    #[bench]
    fn bench_str_sys_gettime_double(b: &mut Bencher) {
        let random_service = Data::new(RandomService::new());
        b.iter(|| {
            random_service.get_ref().str_sys_gettime(64);
            random_service.get_ref().str_sys_gettime(64);
        });
    }

    #[bench]
    fn bench_str_sys_gettime2(b: &mut Bencher) {
        let random_service = Data::new(RandomService::new());
        b.iter(|| {
            random_service.get_ref().str_sys_gettime2(64);
        });
    }
}