use std::collections::HashMap;
use actix_utils::future::{ready, Ready};
use actix_web::{error, Error, FromRequest, HttpRequest};
use actix_web::dev::Payload;
use actix_web::web::Data;
use crate::app::services::translates::Translates;

#[derive(Debug)]
pub struct Lang {
    translates: Data<Translates>
}

impl Lang {
    pub fn new(translates: Data<Translates>) -> Self {
        Self{translates}
    }

    // Return translate value or key
    pub fn translate(&self, lang: &str, key: &str) -> String {
        let mut full_key = lang.to_owned();
        full_key.push('.');
        full_key.push_str(key);

        if let Some(translate) = self.translates.value(&full_key) {
            translate.to_string()
        } else {
            key.to_string()
        }
    }
}

impl FromRequest for Lang {
    type Error = Error;
    type Future = Ready<Result<Lang, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let translates: Option<&Data<Translates>> = req.app_data::<Data<Translates>>();
        if translates.is_none() {
            return ready(Err(error::ErrorInternalServerError("Lang error")));
        }
        let translates = translates.unwrap().to_owned();

        ready(Ok(Lang::new(translates)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate() {
        let translates = Translates::new(HashMap::from([(
            "en.test_key".to_string(),
            "test_value".to_string(),
        )]));
        let lang = Lang::new(Data::new(translates));

        assert_eq!(
            "test_value".to_string(),
            lang.translate("en", "test_key")
        );
        assert_eq!(
            "test_key".to_string(),
            lang.translate("ru", "test_key")
        );
        assert_eq!(
            "test_key123".to_string(),
            lang.translate("en", "test_key123")
        );
    }
}
