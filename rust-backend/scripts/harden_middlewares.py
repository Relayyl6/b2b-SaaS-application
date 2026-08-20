import os

base_path = r"c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend\platform\src\middleware"

request_id_code = """use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use uuid::Uuid;

pub struct RequestId;

impl<S, B> Transform<S, ServiceRequest> for RequestId
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddleware { service }))
    }
}

pub struct RequestIdMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let request_id = req
            .headers()
            .get("X-Request-ID")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| Uuid::parse_str(v).ok())
            .unwrap_or_else(Uuid::new_v4);

        req.extensions_mut().insert(request_id);

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-request-id"),
                actix_web::http::header::HeaderValue::from_str(&request_id.to_string()).unwrap(),
            );
            Ok(res)
        })
    }
}
"""

idempotency_code = """use actix_web::body::EitherBody;
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
            let mut conn = redis_client.get_async_connection().await.map_err(actix_web::error::ErrorInternalServerError)?;
            
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
            
            // If successful, cache it (in a real app, we'd need to extract the body here, 
            // but for foundation stub we just mark the key to prevent immediate replays)
            if res.status().is_success() {
                let _: () = conn.set_ex(&redis_key, "{\"status\": \"cached_success\"}", 86400).await.map_err(actix_web::error::ErrorInternalServerError)?;
            }

            Ok(res.map_into_left_body())
        })
    }
}
"""

rate_limiter_code = """use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, HttpResponse};
use actix_web::body::EitherBody;
use futures_util::future::LocalBoxFuture;
use redis::AsyncCommands;
use std::future::{ready, Ready};
use std::rc::Rc;
use crate::tenant::TenantContext;

pub struct RateLimiter {
    pub redis_client: redis::Client,
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service: Rc::new(service),
            redis_client: self.redis_client.clone(),
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: Rc<S>,
    redis_client: redis::Client,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
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
        let tenant_ctx = req.extensions().get::<TenantContext>().cloned();
        
        let (tenant_id, limit) = match tenant_ctx {
            Some(ctx) => (ctx.tenant_id.to_string(), ctx.tier.monthly_limit()),
            None => {
                // If no tenant context, let it pass (or could block it, but auth middleware handles that)
                let fut = self.service.call(req);
                return Box::pin(async move {
                    let res = fut.await?;
                    Ok(res.map_into_left_body())
                });
            }
        };

        if limit == u64::MAX {
            // Enterprise tier - unlimited
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        let redis_client = self.redis_client.clone();
        let service = self.service.clone();

        Box::pin(async move {
            let mut conn = redis_client.get_async_connection().await.map_err(actix_web::error::ErrorInternalServerError)?;
            let key = format!("rate_limit:{}", tenant_id);
            
            // Simple fixed window for foundation
            let current_count: u64 = conn.incr(&key, 1).await.map_err(actix_web::error::ErrorInternalServerError)?;
            
            if current_count == 1 {
                // Set expiry for 1 month (2592000 seconds)
                let _: () = conn.expire(&key, 2592000).await.map_err(actix_web::error::ErrorInternalServerError)?;
            }

            if current_count > limit {
                let err_res = HttpResponse::TooManyRequests()
                    .json(serde_json::json!({
                        "error": "rate_limit_exceeded",
                        "message": "Monthly API limit exceeded for your pricing tier",
                        "limit": limit
                    }))
                    .map_into_right_body();
                return Ok(req.into_response(err_res));
            }

            let fut = service.call(req);
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}
"""

with open(os.path.join(base_path, "request_id.rs"), "w") as f:
    f.write(request_id_code)

with open(os.path.join(base_path, "idempotency.rs"), "w") as f:
    f.write(idempotency_code)

with open(os.path.join(base_path, "rate_limiter.rs"), "w") as f:
    f.write(rate_limiter_code)

print("Middlewares hardened.")
