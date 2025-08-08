use crate::{Config, LocaleService};
use crate::{Locale, User};
use actix_web::web::Data;
use actix_web::HttpRequest;
use url::Url;

pub struct AppService {
    config: Config,
    url: Url,
    locale_service: Data<LocaleService>,
}

impl AppService {
    pub fn new(config: Config, locale_service: Data<LocaleService>) -> Self {
        let url: Url = Url::parse(&config.app.url).unwrap();
        Self {
            config,
            url,
            locale_service,
        }
    }

    pub fn locale(
        &self,
        req: Option<&HttpRequest>,
        user: Option<&User>,
    ) -> (String, &Locale, &Vec<Locale>) {
        let locale_service = self.locale_service.get_ref();
        let lang: String = locale_service.get_locale_code(req, user);
        let locale: &Locale = locale_service.get_locale_or_default_ref(&lang);
        let locales: &Vec<Locale> = locale_service.get_locales_ref();

        (lang, locale, locales)
    }

    // Return "dark" or "light" or None
    pub fn dark_mode(&self, req: &HttpRequest) -> Option<String> {
        req.cookie(&self.config.app.dark_mode_cookie_key)
            .map(|c| c.value().to_owned())
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}
