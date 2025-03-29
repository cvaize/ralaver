use std::future::{ready, Ready};

use actix_web::http::StatusCode;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;

pub struct ErrorRedirect;

impl<S, B> Transform<S, ServiceRequest> for ErrorRedirect
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ErrorRedirectMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ErrorRedirectMiddleware { service }))
    }
}

pub struct ErrorRedirectMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ErrorRedirectMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res: Self::Response = fut.await?;

            match res.response().error() {
                Some(e) => {
                    if e.as_response_error()
                        .status_code()
                        .eq(&StatusCode::UNAUTHORIZED)
                    {
                        res.response_mut().head_mut().status = StatusCode::SEE_OTHER;

                        res.response_mut().headers_mut().insert(
                            http::header::LOCATION,
                            http::HeaderValue::from_static("/login"),
                        );

                        Ok(res)
                    } else {
                        Ok(res)
                    }
                }
                _ => Ok(res),
            }
        })
    }
}
