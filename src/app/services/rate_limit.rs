use crate::{KeyValueConnection, KeyValueService, TranslatorService};
use actix_web::web::Data;
use actix_web::{error, Error, HttpRequest};
use std::collections::HashMap;
use strum_macros::{Display, EnumString};

pub struct RateLimitService {
    key_value_service: Data<KeyValueService>,
}

impl RateLimitService {
    pub fn new(key_value_service: Data<KeyValueService>) -> Self {
        Self { key_value_service }
    }

    pub fn make_key_from_request(
        &self,
        req: &HttpRequest,
        key: &str,
    ) -> Result<String, RateLimitServiceError> {
        if let Some(val) = req.peer_addr() {
            let mut k = val.ip().to_string();
            k.push('.');
            k.push_str(key);
            return Ok(k);
        };
        Err(RateLimitServiceError::Fail)
    }

    pub fn make_key_from_request_throw_http(
        &self,
        req: &HttpRequest,
        key: &str,
    ) -> Result<String, Error> {
        self.make_key_from_request(req, key)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    fn make_store_key(&self, key: &str) -> String {
        let mut value: String = "rate_limit:".to_string();
        value.push_str(key);
        value
    }

    fn get_connection(&self) -> Result<KeyValueConnection, RateLimitServiceError> {
        self.key_value_service
            .get_ref()
            .get_connection()
            .map_err(|e| {
                log::error!("RateLimitService::get_connection - {e}");
                RateLimitServiceError::Fail
            })
    }

    pub fn clear(&self, key: &str) -> Result<(), RateLimitServiceError> {
        self.key_value_service
            .get_ref()
            .del(self.make_store_key(key))
            .map_err(|e| {
                log::error!("RateLimitService::clear - {e}");
                RateLimitServiceError::Fail
            })
    }

    pub fn clear_throw_http(&self, key: &str) -> Result<(), Error> {
        self.clear(key)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    fn ttl_(
        &self,
        connection: &mut KeyValueConnection,
        key: &str,
    ) -> Result<u64, RateLimitServiceError> {
        let value: i64 = connection.ttl(self.make_store_key(key)).map_err(|e| {
            log::error!("RateLimitService::ttl - {e}");
            RateLimitServiceError::Fail
        })?;
        if value <= 0 {
            return Ok(0);
        }
        Ok(value as u64)
    }

    pub fn ttl(&self, key: &str) -> Result<u64, RateLimitServiceError> {
        let mut connection = self.get_connection()?;
        self.ttl_(&mut connection, key)
    }

    pub fn ttl_message(
        &self,
        translator_service: &TranslatorService,
        lang: &str,
        key: &str,
    ) -> Result<String, RateLimitServiceError> {
        let ttl = self.ttl(key)?;
        let unit = translator_service.choices(&lang, "unit.after_seconds", ttl as i64, None);

        let s = ttl.to_string();
        let mut vars = HashMap::new();
        vars.insert("seconds", s.as_str());
        vars.insert("unit", unit.as_str());

        let message = translator_service.variables(&lang, "validation.rate_limit", &vars);

        Ok(message)
    }

    pub fn ttl_message_throw_http(
        &self,
        translator_service: &TranslatorService,
        lang: &str,
        key: &str,
    ) -> Result<String, Error> {
        self.ttl_message(translator_service, lang, key)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    fn get_(
        &self,
        connection: &mut KeyValueConnection,
        key: &str,
    ) -> Result<u64, RateLimitServiceError> {
        Ok(connection
            .get(self.make_store_key(key))
            .map_err(|e| {
                log::error!("RateLimitService::get - {e}");
                RateLimitServiceError::Fail
            })?
            .unwrap_or(0))
    }

    pub fn get(&self, key: &str) -> Result<u64, RateLimitServiceError> {
        let mut connection = self.get_connection()?;
        self.get_(&mut connection, key)
    }

    fn set_(
        &self,
        connection: &mut KeyValueConnection,
        key: &str,
        amount: u64,
        ttl: u64,
    ) -> Result<(), RateLimitServiceError> {
        connection
            .set_ex(self.make_store_key(key), amount, ttl)
            .map_err(|e| {
                log::error!("RateLimitService::set - {e}");
                RateLimitServiceError::Fail
            })
    }

    pub fn set(&self, key: &str, amount: u64, ttl: u64) -> Result<(), RateLimitServiceError> {
        let mut connection = self.get_connection()?;
        self.set_(&mut connection, key, amount, ttl)
    }

    fn incr_(
        &self,
        connection: &mut KeyValueConnection,
        key: &str,
        amount: u64,
        ttl: u64,
    ) -> Result<u64, RateLimitServiceError> {
        if self.ttl_(connection, key)? == 0 {
            self.set_(connection, key, amount, ttl)?;
            return Ok(amount);
        }

        Ok(connection
            .incr(self.make_store_key(key), amount)
            .map_err(|e| {
                log::error!("RateLimitService::incr - {e}");
                RateLimitServiceError::Fail
            })?)
    }

    pub fn incr(&self, key: &str, amount: u64, ttl: u64) -> Result<u64, RateLimitServiceError> {
        let mut connection = self.get_connection()?;
        self.incr_(&mut connection, key, amount, ttl)
    }

    fn remaining_(
        &self,
        connection: &mut KeyValueConnection,
        key: &str,
        max_attempts: u64,
    ) -> Result<u64, RateLimitServiceError> {
        let value = self.get_(connection, key)?;
        if value >= max_attempts {
            return Ok(0);
        }
        Ok(max_attempts - value)
    }

    pub fn remaining(&self, key: &str, max_attempts: u64) -> Result<u64, RateLimitServiceError> {
        let mut connection = self.get_connection()?;
        self.remaining_(&mut connection, key, max_attempts)
    }

    fn is_too_many_attempts_(
        &self,
        connection: &mut KeyValueConnection,
        key: &str,
        max_attempts: u64,
    ) -> Result<bool, RateLimitServiceError> {
        Ok(self.get_(connection, key)? >= max_attempts)
    }

    pub fn is_too_many_attempts(
        &self,
        key: &str,
        max_attempts: u64,
    ) -> Result<bool, RateLimitServiceError> {
        let mut connection = self.get_connection()?;
        self.is_too_many_attempts_(&mut connection, key, max_attempts)
    }

    pub fn attempt(
        &self,
        key: &str,
        max_attempts: u64,
        ttl: u64,
    ) -> Result<bool, RateLimitServiceError> {
        let mut connection = self.get_connection()?;
        if self.is_too_many_attempts_(&mut connection, key, max_attempts)? {
            return Ok(false);
        }
        Ok(self.incr_(&mut connection, key, 1, ttl)? <= max_attempts)
    }

    pub fn attempt_throw_http(
        &self,
        key: &str,
        max_attempts: u64,
        ttl: u64,
    ) -> Result<bool, Error> {
        self.attempt(key, max_attempts, ttl)
            .map_err(|_| error::ErrorInternalServerError(""))
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum RateLimitServiceError {
    Fail,
}

#[cfg(test)]
mod tests {
    use crate::preparation;
    use test::Bencher;

    static KEY: &str = "172.18.0.1";
    static MAX_ATTEMPTS: u64 = 5;
    // 600 secs = 10 minutes
    static TTL: u64 = 600;

    #[test]
    fn test() {
        let (_, all_services) = preparation();
        let rate_limit_service = all_services.rate_limit_service.get_ref();

        rate_limit_service.clear(KEY).unwrap();
        for i in 0..MAX_ATTEMPTS {
            let remaining = rate_limit_service.remaining(KEY, MAX_ATTEMPTS).unwrap();
            assert_eq!(remaining, MAX_ATTEMPTS - i);
            let executed = rate_limit_service.attempt(KEY, MAX_ATTEMPTS, TTL).unwrap();
            assert!(executed);
        }
        let executed = rate_limit_service.attempt(KEY, MAX_ATTEMPTS, TTL).unwrap();
        assert!(!executed);
        assert!(rate_limit_service
            .is_too_many_attempts(KEY, MAX_ATTEMPTS)
            .unwrap());
        rate_limit_service.clear(KEY).unwrap();
    }

    #[bench]
    fn bench_test(b: &mut Bencher) {
        let (_, all_services) = preparation();
        let rate_limit_service = all_services.rate_limit_service.get_ref();

        b.iter(|| {
            rate_limit_service.clear(KEY).unwrap();
            rate_limit_service.attempt(KEY, MAX_ATTEMPTS, TTL).unwrap();
        });
    }
}
