use crate::{Config, Debug, DebugService};
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
pub struct DebugMiddleware;

impl<S, B> Transform<S, ServiceRequest> for DebugMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = InnerDebugMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(InnerDebugMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct InnerDebugMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for InnerDebugMiddleware<S>
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
        let cookie_data = process_debug(&mut req);
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

pub fn process_debug(req: &mut ServiceRequest) -> Result<(String, String, i64, bool), Error> {
    let config: Option<&Data<Config>> = req.app_data::<Data<Config>>();
    if config.is_none() {
        return Err(error::ErrorInternalServerError("Config error"));
    }
    let config = config.unwrap().get_ref();

    let debug_service: Option<&Data<DebugService>> = req.app_data::<Data<DebugService>>();
    if debug_service.is_none() {
        return Err(error::ErrorInternalServerError("DebugService error"));
    }
    let debug_service = debug_service.unwrap().get_ref();

    let cookie_key = match req.cookie(&config.debug.key) {
        Some(cookie_key) => Some(cookie_key.value().to_string()),
        _ => None,
    };

    let (new_value, debug): (String, Debug) = debug_service
        .renew(cookie_key)
        .map_err(|_| error::ErrorInternalServerError("DebugService error"))?;

    req.extensions_mut().insert(debug);

    let secure = config.debug.secure;
    Ok((
        config.debug.key.to_owned(),
        new_value,
        config.debug.expires as i64,
        secure
    ))
}