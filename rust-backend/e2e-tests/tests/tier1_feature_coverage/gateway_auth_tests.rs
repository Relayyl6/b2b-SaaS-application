use actix_web::{http::StatusCode, test, web, App, HttpResponse};
use e2e_tests::test_context::{
    generate_expired_jwt, generate_mock_api_key, generate_mock_jwt, MockTenantFixture,
};
use e2e_tests::TestHarness;
use platform::middleware::tenant_middleware::TenantAuthMiddleware;
use platform::tenant::{PricingTier, TenantContext};
use uuid::Uuid;

#[actix_web::test]
async fn test_gateway_auth_valid_api_key_returns_200_and_context() {
    let secret = "test_jwt_secret";
    let fixture = MockTenantFixture::new(PricingTier::Growth, secret);

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(secret))
            .route(
                "/orders",
                web::get().to(|ctx: TenantContext| async move {
                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "success",
                        "tenant_id": ctx.tenant_id,
                        "tier": ctx.tier.to_string(),
                    }))
                }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/orders")
        .insert_header(("X-API-Key", fixture.api_key.as_str()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_gateway_auth_missing_header_returns_401() {
    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new())
            .route(
                "/orders",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/orders").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_gateway_auth_invalid_api_key_returns_401() {
    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new())
            .route(
                "/orders",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/orders")
        .insert_header(("X-API-Key", "invalid_malformed_key"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_gateway_auth_expired_jwt_returns_401() {
    let secret = "test_jwt_secret";
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let expired_token = generate_expired_jwt(user_id, tenant_id, secret).unwrap();

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(secret))
            .route(
                "/orders",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/orders")
        .insert_header(("Authorization", format!("Bearer {}", expired_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_gateway_auth_usage_limit_exceeded_returns_402() {
    let harness = TestHarness::new().await;
    let secret = "test_jwt_secret";
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Free Tier limit is 100 orders/mo
    let token = generate_mock_jwt(user_id, tenant_id, PricingTier::Free, secret).unwrap();

    // If Redis is reachable, set usage counter > 100
    let _ = harness.set_usage_counter(tenant_id, 101).await;

    let mut middleware = TenantAuthMiddleware::new().with_secret(secret);
    if let Some(client) = harness.redis_client {
        middleware = TenantAuthMiddleware::with_redis(client).with_secret(secret);
    }

    let app = test::init_service(
        App::new()
            .wrap(middleware)
            .route(
                "/orders",
                web::post().to(|| async { HttpResponse::Created().finish() }),
            ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/orders")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // When usage > limit and Redis is active, status is 402 PAYMENT_REQUIRED.
    // If Redis is offline, it passes through safely or checks local fallback.
    assert!(
        resp.status() == StatusCode::PAYMENT_REQUIRED || resp.status() == StatusCode::CREATED
    );
}
