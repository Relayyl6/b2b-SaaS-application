use actix_web::body::EitherBody;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, HttpResponse};
use futures_util::future::LocalBoxFuture;
use redis::AsyncCommands;
use std::future::{ready, Ready};
use std::rc::Rc;
use crate::tenant::TenantContext;

pub struct Idempotency {
    pub redis_client: redis::Client,
}

impl<S, B> Transform<S, ServiceRequest> for Idempotency
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = IdempotencyMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(IdempotencyMiddleware {
            service: Rc::new(service),
            redis_client: self.redis_client.clone(),
        }))
    }
}

pub struct IdempotencyMiddleware<S> {
    service: Rc<S>,
    redis_client: redis::Client,
}

impl<S, B> Service<ServiceRequest> for IdempotencyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let idempotency_key = req.headers().get("Idempotency-Key").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        
        let method = req.method().clone();
        if method == actix_web::http::Method::GET || method == actix_web::http::Method::DELETE || idempotency_key.is_none() {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        let idempotency_key = idempotency_key.unwrap();
        
        // Need TenantContext to namespace the key
        let tenant_id = match req.extensions().get::<TenantContext>() {
            Some(ctx) => ctx.tenant_id.to_string(),
            None => "global".to_string(),
        };

        let redis_key = format!("idemp:{}:{}", tenant_id, idempotency_key);
        let redis_client = self.redis_client.clone();
        let service = self.service.clone();

        Box::pin(async move {
            let mut conn = redis_client.get_multiplexed_async_connection().await.map_err(actix_web::error::ErrorInternalServerError)?;
            
            // Check if key exists
            let cached_response: Option<String> = conn.get(&redis_key).await.map_err(actix_web::error::ErrorInternalServerError)?;
            
            if let Some(cached) = cached_response {
                let res = HttpResponse::Ok()
                    .content_type("application/json")
                    .body(cached)
                    .map_into_right_body();
                return Ok(req.into_response(res));
            }

            // Proceed with normal request
            let fut = service.call(req);
            let res = fut.await?;
            
            // If successful, cache it
            if res.status().is_success() {
                let _: () = conn.set_ex(&redis_key, "{\"status\": \"cached_success\"}", 86400).await.map_err(actix_web::error::ErrorInternalServerError)?;
            }

            Ok(res.map_into_left_body())
        })
    }
}
