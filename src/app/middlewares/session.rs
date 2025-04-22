use crate::{Config, Session, SessionService};
use actix_utils::future::{ready, Ready};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::web::Data;
use actix_web::{
    body::MessageBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error, Error, HttpMessage,
};
use std::{future::Future, pin::Pin, rc::Rc};

#[derive(Clone)]
pub struct SessionMiddleware;

impl<S, B> Transform<S, ServiceRequest> for SessionMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = InnerSessionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(InnerSessionMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct InnerSessionMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for InnerSessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        // A more complex middleware, could return an error or an early response here.
        let cookie_data = process_session(&mut req);
        if let Err(e) = cookie_data {
            return Box::pin(async move { Err(e) });
        }
        let (cookie_key, cookie_value, expires, secure) = cookie_data.unwrap();

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            let cookie = Cookie::build(cookie_key, cookie_value)
                .path("/")
                .http_only(true)
                .secure(secure)
                .max_age(Duration::seconds(expires))
                .finish();
            res.response_mut().add_cookie(&cookie).unwrap();

            Ok(res)
        })
    }
}

pub fn process_session(req: &mut ServiceRequest) -> Result<(String, String, i64, bool), Error> {
    let config: Option<&Data<Config>> = req.app_data::<Data<Config>>();
    if config.is_none() {
        return Err(error::ErrorInternalServerError("Config error"));
    }
    let config = config.unwrap().get_ref();

    let session_service: Option<&Data<SessionService>> = req.app_data::<Data<SessionService>>();
    if session_service.is_none() {
        return Err(error::ErrorInternalServerError("SessionService error"));
    }
    let session_service = session_service.unwrap().get_ref();

    let cookie_key = match req.cookie(&config.session.key) {
        Some(cookie_key) => Some(cookie_key.value().to_string()),
        _ => None,
    };

    let (new_value, session): (String, Session) = session_service
        .renew(cookie_key)
        .map_err(|_| error::ErrorInternalServerError("SessionService error"))?;

    req.extensions_mut().insert(session);

    let secure = config.session.secure;
    Ok((
        config.session.key.to_owned(),
        new_value,
        config.session.expires as i64,
        secure
    ))
}
