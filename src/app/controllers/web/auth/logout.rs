use crate::app::controllers::web::auth::login::locale;
use crate::{log_map_err, FlashService, Session};
use crate::{
    Alert, AppService, AuthService, Translator, TranslatorService,
    ALERTS_KEY,
};
use actix_web::web::Data;
use actix_web::web::Redirect;
use actix_web::{error, Error, HttpRequest, Responder, Result};

pub async fn invoke(
    req: HttpRequest,
    session: Session,
    flash_service: Data<FlashService>,
    auth_service: Data<AuthService<'_>>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<impl Responder, Error> {
    let flash_service = flash_service.get_ref();
    let auth_service = auth_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();

    let user = auth_service.authenticate_by_session(&session);
    auth_service
        .logout_from_session(&session)
        .map_err(log_map_err!(
            error::ErrorInternalServerError("AuthService error"),
            "Logout:invoke"
        ))?;

    let (lang, _, _) = locale(&user, app_service, &req, &session);

    let translator = Translator::new(&lang, translator_service);
    let alert_str = translator.simple("auth.alert.logout.success");

    let alerts = vec![Alert::success(alert_str)];

    flash_service.save_throw_http(&session, ALERTS_KEY, &alerts)?;

    Ok(Redirect::to("/login").see_other())
}
