use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
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
            let mut conn = redis_client.get_multiplexed_async_connection().await.map_err(actix_web::error::ErrorInternalServerError)?;
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
