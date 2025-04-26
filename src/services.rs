use crate::connections::Connections;
use crate::{AppService, AuthService, Config, CryptService, HashService, KeyValueService, LocaleService, MailService, RandomService, RateLimitService, TemplateService, TranslatorService, UserService};
use actix_web::web::Data;

pub struct BaseServices {
    pub config: Data<Config>,
}

pub fn base(config: Config) -> BaseServices {
    BaseServices {
        config: Data::new(config),
    }
}

pub struct AdvancedServices <'a>{
    pub key_value: Data<KeyValueService>,
    pub translator: Data<TranslatorService>,
    pub template: Data<TemplateService>,
    pub crypt: Data<CryptService<'a>>,
    pub hash: Data<HashService<'a>>,
    pub auth: Data<AuthService<'a>>,
    pub locale: Data<LocaleService>,
    pub app: Data<AppService>,
    pub mail: Data<MailService>,
    pub rand: Data<RandomService>,
    pub rate_limit: Data<RateLimitService>,
    pub user: Data<UserService>,
}

pub fn advanced<'a>(c: &Connections, s: &BaseServices) -> AdvancedServices <'a>{
    let user = Data::new(UserService::new(c.mysql.clone()));
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

    let hash = Data::new(HashService::new());
    let crypt = Data::new(CryptService::new(s.config.clone(), rand.clone(), hash.clone()));
    let auth = Data::new(AuthService::new(
        s.config.clone(),
        c.mysql.clone(),
        hash.clone(),
        key_value.clone(),
        user.clone(),
        rand.clone(),
        crypt.clone(),
    ));
    let locale = Data::new(LocaleService::new(s.config.clone()));
    let app = Data::new(AppService::new(s.config.clone(), locale.clone()));
    let mail = Data::new(MailService::new(s.config.clone(), c.smtp.clone()));
    let rate_limit = Data::new(RateLimitService::new(key_value.clone()));

    AdvancedServices {
        key_value,
        translator,
        template,
        hash,
        crypt,
        auth,
        locale,
        app,
        mail,
        rand,
        rate_limit,
        user,
    }
}

#[allow(dead_code)]
pub struct Services <'a>{
    pub config: Data<Config>,
    pub key_value: Data<KeyValueService>,
    pub translator: Data<TranslatorService>,
    pub template: Data<TemplateService>,
    pub hash: Data<HashService<'a>>,
    pub auth: Data<AuthService<'a>>,
    pub crypt: Data<CryptService<'a>>,
    pub locale: Data<LocaleService>,
    pub app: Data<AppService>,
    pub mail: Data<MailService>,
    pub rand: Data<RandomService>,
    pub rate_limit: Data<RateLimitService>,
    pub user: Data<UserService>,
}

pub fn join_to_all<'a>(base: BaseServices, advanced: AdvancedServices<'a>) -> Services<'a> {
    Services {
        config: base.config,
        key_value: advanced.key_value,
        translator: advanced.translator,
        template: advanced.template,
        hash: advanced.hash,
        auth: advanced.auth,
        crypt: advanced.crypt,
        locale: advanced.locale,
        app: advanced.app,
        mail: advanced.mail,
        rand: advanced.rand,
        rate_limit: advanced.rate_limit,
        user: advanced.user,
    }
}
