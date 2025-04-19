use crate::Session;
use crate::{
    Alert, AppService, AuthService, KeyValueService, SessionService, Translator, TranslatorService,
    ALERTS_KEY,
};
use actix_web::web::Data;
use actix_web::web::Redirect;
use actix_web::{error, Error, HttpRequest, Responder, Result};

pub async fn invoke(
    req: HttpRequest,
    session: Session,
    auth_service: Data<AuthService<'_>>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    session_service: Data<SessionService>,
    key_value_service: Data<KeyValueService>,
) -> Result<impl Responder, Error> {
    let auth_service = auth_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();
    let session_service = session_service.get_ref();
    let key_value_service = key_value_service.get_ref();

    let user = auth_service.authenticate_by_session(&session);
    auth_service.logout_from_session(&session);

    let (lang, _, _) = match user {
        Ok(user) => app_service.locale(Some(&req), Some(&session), Some(&user)),
        _ => app_service.locale(Some(&req), Some(&session), None),
    };

    let translator = Translator::new(&lang, translator_service);
    let alert_str = translator.simple("auth.alert.logout.success");

    let alerts = vec![Alert::success(alert_str)];

    let key = session_service.make_session_data_key(&session, ALERTS_KEY);
    key_value_service
        .set_ex(key, &alerts, 600)
        .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?;

    Ok(Redirect::to("/login").see_other())
}
