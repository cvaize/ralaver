use crate::{Alert, AlertService, AppService, AuthService, Translator, TranslatorService};
use actix_session::Session;
use actix_web::web::Data;
use actix_web::web::Redirect;
use actix_web::{error, Error, HttpRequest, Responder, Result};

pub async fn invoke(
    req: HttpRequest,
    session: Session,
    auth: Data<AuthService<'_>>,
    alert_service: Data<AlertService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<impl Responder, Error> {
    let user = auth.get_ref().authenticate_by_session(&session);
    auth.logout_from_session(&session);

    let (lang, _, _) = match user {
        Ok(user) => app_service.locale(Some(&req), Some(&session), Some(&user)),
        _ => app_service.locale(Some(&req), Some(&session), None),
    };

    let translator = Translator::new(&lang, translator_service.get_ref());
    let alert_str = translator.simple("auth.alert.logout.success");

    let alerts = vec![Alert::success(alert_str)];
    alert_service
        .get_ref()
        .insert_into_session(&session, &alerts)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    Ok(Redirect::to("/login").see_other())
}
