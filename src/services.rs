use crate::connections::Connections;
use crate::{
    AppService, AuthService, Config, HashService, KeyValueService, LocaleService, MailService,
    RandomService, SessionService, TemplateService, TranslatorService,
};
use actix_web::web::Data;
use argon2::Argon2;

pub struct BaseServices {
    pub config: Data<Config>,
}

pub fn base(config: Config) -> BaseServices {
    BaseServices {
        config: Data::new(config),
    }
}

pub struct AdvancedServices<'a> {
    pub key_value: Data<KeyValueService>,
    pub translator: Data<TranslatorService>,
    pub template: Data<TemplateService>,
    pub session: Data<SessionService>,
    pub hash: Data<HashService<'a>>,
    pub auth: Data<AuthService<'a>>,
    pub locale: Data<LocaleService>,
    pub app: Data<AppService>,
    pub mail: Data<MailService>,
    pub rand: Data<RandomService>,
}

pub fn advanced<'a>(c: &Connections, s: &BaseServices) -> AdvancedServices<'a> {
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
    let session = Data::new(SessionService::new(
        s.config.clone(),
        key_value.clone(),
        rand.clone(),
    ));
    let hash = Data::new(HashService::new(Argon2::default()));
    let auth = Data::new(AuthService::new(
        s.config.clone(),
        c.mysql.clone(),
        hash.clone(),
        session.clone(),
        key_value.clone(),
    ));
    let locale = Data::new(LocaleService::new(
        s.config.clone(),
        session.clone(),
        key_value.clone(),
    ));
    let app = Data::new(AppService::new(s.config.clone(), locale.clone()));
    let mail = Data::new(MailService::new(s.config.clone(), c.smtp.clone()));

    AdvancedServices {
        key_value,
        translator,
        template,
        session,
        hash,
        auth,
        locale,
        app,
        mail,
        rand,
    }
}

#[allow(dead_code)]
pub struct Services<'a> {
    pub config: Data<Config>,
    pub key_value: Data<KeyValueService>,
    pub translator: Data<TranslatorService>,
    pub template: Data<TemplateService>,
    pub session: Data<SessionService>,
    pub hash: Data<HashService<'a>>,
    pub auth: Data<AuthService<'a>>,
    pub locale: Data<LocaleService>,
    pub app: Data<AppService>,
    pub mail: Data<MailService>,
    pub rand: Data<RandomService>,
}

pub fn join_to_all(base: BaseServices, advanced: AdvancedServices) -> Services {
    Services {
        config: base.config,
        key_value: advanced.key_value,
        translator: advanced.translator,
        template: advanced.template,
        session: advanced.session,
        hash: advanced.hash,
        auth: advanced.auth,
        locale: advanced.locale,
        app: advanced.app,
        mail: advanced.mail,
        rand: advanced.rand,
    }
}
