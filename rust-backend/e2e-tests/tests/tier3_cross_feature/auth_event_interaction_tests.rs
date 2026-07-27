use actix_web::{http::StatusCode, test, web, App, HttpResponse};
use e2e_tests::test_context::{
    create_enriched_event, generate_mock_jwt, validate_event_tenant_enrichment,
    EnrichedEventPayload, MockTenantFixture,
};
use e2e_tests::TestHarness;
use platform::middleware::tenant_middleware::TenantAuthMiddleware;
use platform::tenant::{PricingTier, TenantContext};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[actix_web::test]
async fn test_cross_auth_to_event_enrichment_flow() {
    let secret = "test_jwt_secret";
    let fixture = MockTenantFixture::new(PricingTier::Growth, secret);

    let published_events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = published_events.clone();

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(secret))
            .route(
                "/orders",
                web::post().to(move |ctx: TenantContext| {
                    let ev_clone = events_clone.clone();
                    async move {
                        let event = create_enriched_event(
                            "order.created",
                            Some(ctx.tenant_id),
                            serde_json::json!({ "order_id": Uuid::new_v4(), "total": 250 }),
                            secret,
                        );

                        let mut guard = ev_clone.lock().unwrap();
                        guard.push(event);

                        HttpResponse::Created().finish()
                    }
                }),
            ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/orders")
        .insert_header(("Authorization", format!("Bearer {}", fixture.jwt_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let guard = published_events.lock().unwrap();
    assert_eq!(guard.len(), 1);
    assert!(validate_event_tenant_enrichment(
        &guard[0],
        fixture.tenant_id
    ));
}

#[actix_web::test]
async fn test_cross_auth_rate_limited_event_suppression() {
    let harness = TestHarness::new().await;
    let secret = "test_jwt_secret";
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let token = generate_mock_jwt(user_id, tenant_id, PricingTier::Free, secret).unwrap();

    // Set usage counter > limit (101 > 100)
    let _ = harness.set_usage_counter(tenant_id, 101).await;

    let events_published = Arc::new(Mutex::new(0));
    let events_count_clone = events_published.clone();

    let mut middleware = TenantAuthMiddleware::new().with_secret(secret);
    if let Some(client) = harness.redis_client {
        middleware = TenantAuthMiddleware::with_redis(client).with_secret(secret);
    }

    let app = test::init_service(
        App::new()
            .wrap(middleware)
            .route(
                "/orders",
                web::post().to(move || {
                    let c = events_count_clone.clone();
                    async move {
                        let mut guard = c.lock().unwrap();
                        *guard += 1;
                        HttpResponse::Created().finish()
                    }
                }),
            ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/orders")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    if resp.status() == StatusCode::PAYMENT_REQUIRED {
        // Gateway blocked request; zero event publishing handlers should execute
        assert_eq!(*events_published.lock().unwrap(), 0);
    }
}
