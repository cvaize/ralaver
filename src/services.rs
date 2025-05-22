use crate::connections::Connections;
use crate::{AppService, AuthService, Config, CryptService, HashService, KeyValueService, LocaleService, MailService, RandomService, RateLimitService, TemplateService, TranslatorService, UserService, WebAuthService};
use actix_web::web::Data;
use crate::app::repositories::UserRepository;

pub struct BaseServices {
    pub config: Data<Config>,
}

pub fn base(config: Config) -> BaseServices {
    BaseServices {
        config: Data::new(config),
    }
}

pub struct AdvancedServices {
    pub key_value: Data<KeyValueService>,
    pub translator: Data<TranslatorService>,
    pub template: Data<TemplateService>,
    pub crypt: Data<CryptService>,
    pub hash: Data<HashService>,
    pub web_auth: Data<WebAuthService>,
    pub auth: Data<AuthService>,
    pub locale: Data<LocaleService>,
    pub app: Data<AppService>,
    pub mail: Data<MailService>,
    pub rand: Data<RandomService>,
    pub rate_limit: Data<RateLimitService>,
    pub user: Data<UserService>,
    pub user_rep: Data<UserRepository>,
}

pub fn advanced(c: &Connections, s: &BaseServices) -> AdvancedServices {
    let key_value = Data::new(KeyValueService::new(c.redis.clone()));
    let translator = Data::new(
        TranslatorService::new_from_files(s.config.clone())
            .expect("Fail init TranslatorService::new_from_files"),
    );
    let template = Data::new(
        TemplateService::new_from_files(s.config.clone())
            .expect("Fail init TemplateService::new_from_files"),
    );
    let rand = Data::new(RandomService::new());

    let hash = Data::new(HashService::new(s.config.clone()));
    let user_rep = Data::new(UserRepository::new(c.mysql.clone()));
    let user = Data::new(UserService::new(hash.clone(), user_rep.clone()));

    let crypt = Data::new(CryptService::new(
        s.config.clone(),
        rand.clone(),
        hash.clone(),
    ));
    let auth = Data::new(AuthService::new(
        key_value.clone(),
        hash.clone(),
        user.clone(),
        user_rep.clone(),
    ));
    let locale = Data::new(LocaleService::new(s.config.clone()));
    let app = Data::new(AppService::new(s.config.clone(), locale.clone()));
    let mail = Data::new(MailService::new(s.config.clone(), c.smtp.clone()));
    let rate_limit = Data::new(RateLimitService::new(key_value.clone()));
    let web_auth = Data::new(WebAuthService::new(
        s.config.clone(),
        crypt.clone(),
        rand.clone(),
        key_value.clone(),
        hash.clone(),
        user.clone(),
    ));

    AdvancedServices {
        key_value,
        translator,
        template,
        hash,
        crypt,
        web_auth,
        auth,
        locale,
        app,
        mail,
        rand,
        rate_limit,
        user,
        user_rep,
    }
}

pub struct Services {
    pub config: Data<Config>,
    pub key_value: Data<KeyValueService>,
    pub translator: Data<TranslatorService>,
    pub template: Data<TemplateService>,
    pub hash: Data<HashService>,
    pub web_auth: Data<WebAuthService>,
    pub auth: Data<AuthService>,
    pub crypt: Data<CryptService>,
    pub locale: Data<LocaleService>,
    pub app: Data<AppService>,
    pub mail: Data<MailService>,
    pub rand: Data<RandomService>,
    pub rate_limit: Data<RateLimitService>,
    pub user: Data<UserService>,
    pub user_rep: Data<UserRepository>,
}

pub fn join_to_all(base: BaseServices, advanced: AdvancedServices) -> Services {
    Services {
        config: base.config,
        key_value: advanced.key_value,
        translator: advanced.translator,
        template: advanced.template,
        hash: advanced.hash,
        web_auth: advanced.web_auth,
        auth: advanced.auth,
        crypt: advanced.crypt,
        locale: advanced.locale,
        app: advanced.app,
        mail: advanced.mail,
        rand: advanced.rand,
        rate_limit: advanced.rate_limit,
        user: advanced.user,
        user_rep: advanced.user_rep,
    }
}
