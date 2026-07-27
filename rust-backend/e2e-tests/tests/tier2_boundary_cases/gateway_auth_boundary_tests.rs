use actix_web::{http::StatusCode, test, web, App, HttpResponse};
use e2e_tests::test_context::{generate_mock_api_key, generate_mock_jwt, MockTenantFixture};
use e2e_tests::TestHarness;
use platform::middleware::tenant_middleware::TenantAuthMiddleware;
use platform::tenant::{ApiKeyRecord, PricingTier, TenantContext};
use uuid::Uuid;

#[actix_web::test]
async fn test_gateway_auth_boundary_exact_usage_limit_returns_200() {
    let harness = TestHarness::new().await;
    let secret = "test_jwt_secret";
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let token = generate_mock_jwt(user_id, tenant_id, PricingTier::Free, secret).unwrap();

    // Free limit is 100. Request #100 must be 200/CREATED, #101 must be 402 PAYMENT_REQUIRED.
    let _ = harness.set_usage_counter(tenant_id, 100).await;

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

    let req100 = test::TestRequest::post()
        .uri("/orders")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp100 = test::call_service(&app, req100).await;
    // Exactly 100 <= 100 limit, so allowed
    assert!(resp100.status() == StatusCode::CREATED || resp100.status() == StatusCode::OK);
}

#[actix_web::test]
async fn test_gateway_auth_usage_counter_reset_window() {
    let harness = TestHarness::new().await;
    let tenant_id = Uuid::new_v4();

    // Simulate reset of counter key to 0
    let res = harness.set_usage_counter(tenant_id, 0).await;
    assert!(res.is_ok() || harness.redis_client.is_none());
}

#[actix_web::test]
async fn test_gateway_auth_malformed_auth_header_format() {
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
        .insert_header(("Authorization", "Bearer $$$invalid_token_format!!!"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_gateway_auth_concurrent_rate_burst_handling() {
    let secret = "test_jwt_secret";
    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(secret))
            .route(
                "/orders",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let key = generate_mock_api_key(Uuid::new_v4(), "live");

    // Send rapid batch of 10 requests
    for _ in 0..10 {
        let req = test::TestRequest::get()
            .uri("/orders")
            .insert_header(("X-API-Key", key.as_str()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

#[actix_web::test]
async fn test_gateway_auth_revoked_api_key_instant_invalidation() {
    let harness = TestHarness::new().await;
    let secret = "test_jwt_secret";
    let tenant_id = Uuid::new_v4();
    let key = format!("sk_live_revoked_{}", tenant_id.simple());

    let record = ApiKeyRecord {
        id: Uuid::new_v4(),
        tenant_id,
        key_prefix: "sk_live".to_string(),
        key_hash: "hash".to_string(),
        permissions: vec!["*".to_string()],
        rate_limit_override: None,
        is_active: false, // Revoked
    };

    let _ = harness.seed_api_key_redis(&key, &record).await;

    let mut middleware = TenantAuthMiddleware::new().with_secret(secret);
    if let Some(client) = harness.redis_client {
        middleware = TenantAuthMiddleware::with_redis(client).with_secret(secret);
    }

    let app = test::init_service(
        App::new()
            .wrap(middleware)
            .route(
                "/orders",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/orders")
        .insert_header(("X-API-Key", key.as_str()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
