use crate::{AuthService, AuthServiceError, AuthToken, Session, User};
use actix_utils::future::{ready, Ready};
use actix_web::body::BoxBody;
use actix_web::web::Data;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpRequest, HttpResponse,
};
use http::{HeaderValue, StatusCode};
use std::sync::Arc;
use std::{future::Future, pin::Pin, rc::Rc};

#[derive(Clone)]
pub struct WebAuthMiddleware;

static REDIRECT_TO: &str = "/login";

impl<S> Transform<S, ServiceRequest> for WebAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = InnerWebAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(InnerWebAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct InnerWebAuthMiddleware<S> {
    service: Rc<S>,
}

fn unauthorized_redirect(auth_service: &AuthService) -> HttpResponse {
    HttpResponse::SeeOther()
        .cookie(auth_service.make_auth_token_clear_cookie())
        .insert_header((
            http::header::LOCATION,
            HeaderValue::from_static(REDIRECT_TO),
        ))
        .finish()
}

impl<S> Service<ServiceRequest> for InnerWebAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_service: &Data<AuthService<'_>> = req.app_data::<Data<AuthService<'_>>>().unwrap();
        let auth_service = Arc::clone(auth_service);

        let old_auth_token = auth_service.get_auth_token_from_request(req.request());

        if old_auth_token.is_none() {
            return Box::pin(async move {
                let res = unauthorized_redirect(auth_service.as_ref());
                Ok(req.into_response(res))
            });
        }
        let old_auth_token: Rc<AuthToken> = Rc::new(old_auth_token.unwrap());

        let auth_data = auth_service.login_by_auth_token(old_auth_token.as_ref());

        if auth_data.is_err() {
            return Box::pin(async move {
                let res = unauthorized_redirect(auth_service.as_ref());
                Ok(req.into_response(res))
            });
        }

        let (user, new_auth_token) = auth_data.unwrap();
        let new_auth_token: Rc<AuthToken> = Rc::new(new_auth_token);
        let new_auth_token_rc: Rc<AuthToken> = Rc::clone(&new_auth_token);

        let user_rc: Rc<User> = Rc::new(user);
        req.extensions_mut().insert(Rc::clone(&user_rc));

        let session_rc: Rc<Session> = Rc::new(Session::new(
            Rc::clone(&old_auth_token),
            Rc::clone(&new_auth_token),
        ));
        req.extensions_mut().insert(Rc::clone(&session_rc));

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res: Self::Response = fut.await?;

            match res.response().error() {
                Some(e) => {
                    // TODO: Render error page
                    if e.as_response_error()
                        .status_code()
                        .eq(&StatusCode::UNAUTHORIZED)
                    {
                        let res_mut = res.response_mut();

                        res_mut.head_mut().status = StatusCode::SEE_OTHER;

                        res_mut.headers_mut().insert(
                            http::header::LOCATION,
                            HeaderValue::from_static(REDIRECT_TO),
                        );

                        let cookie = auth_service.make_auth_token_clear_cookie();
                        let _ = auth_service.expire_auth_token(new_auth_token_rc.as_ref());
                        let _ = res_mut.add_cookie(&cookie);

                        Ok(res)
                    } else {
                        Ok(res)
                    }
                }
                _ => {
                    let cookie = auth_service
                        .make_auth_token_cookie_throw_http(new_auth_token_rc.as_ref())?;
                    res.response_mut().add_cookie(&cookie).unwrap();

                    Ok(res)
                }
            }
        })
    }
}
