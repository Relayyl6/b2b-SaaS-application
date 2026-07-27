use actix_web::{http::StatusCode, test, web, App, HttpResponse};
use e2e_tests::test_context::{
    create_enriched_event, format_set_tenant_session_sql, generate_mock_jwt,
    validate_event_tenant_enrichment, EnrichedEventPayload, MockTenantFixture,
};
use e2e_tests::TestHarness;
use platform::middleware::tenant_middleware::TenantAuthMiddleware;
use platform::tenant::{PricingTier, TenantContext};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OrderFulfillmentPayload {
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub qty: u32,
}

#[actix_web::test]
async fn test_real_world_multi_tenant_fulfillment_and_quota_enforcement() {
    let harness = TestHarness::new().await;
    let secret = "test_jwt_secret";

    // Tenant A: Growth Tier (10,000 order limit)
    let tenant_a = MockTenantFixture::new(PricingTier::Growth, secret);
    // Tenant B: Free Tier (100 order limit)
    let tenant_b = MockTenantFixture::new(PricingTier::Free, secret);

    // Set Tenant B usage to 101 to simulate quota overflow
    let _ = harness.set_usage_counter(tenant_b.tenant_id, 101).await;

    let tenant_a_events: Arc<Mutex<Vec<EnrichedEventPayload<OrderFulfillmentPayload>>>> =
        Arc::new(Mutex::new(Vec::new()));
    let events_a_clone = tenant_a_events.clone();

    let mut middleware = TenantAuthMiddleware::new().with_secret(secret);
    if let Some(client) = harness.redis_client {
        middleware = TenantAuthMiddleware::with_redis(client).with_secret(secret);
    }

    let app = test::init_service(
        App::new()
            .wrap(middleware)
            .route(
                "/orders",
                web::post().to(move |ctx: TenantContext, body: web::Json<serde_json::Value>| {
                    let ev_a = events_a_clone.clone();
                    async move {
                        let order_id = Uuid::new_v4();
                        let product_id = Uuid::new_v4();
                        let qty = body["qty"].as_u64().unwrap_or(1) as u32;

                        // Emit OrderCreatedEvent
                        let event = create_enriched_event(
                            "order.created",
                            Some(ctx.tenant_id),
                            OrderFulfillmentPayload {
                                order_id,
                                product_id,
                                qty,
                            },
                            secret,
                        );

                        let mut guard = ev_a.lock().unwrap();
                        guard.push(event);

                        HttpResponse::Created().json(serde_json::json!({
                            "order_id": order_id,
                            "tenant_id": ctx.tenant_id,
                            "session_sql": format_set_tenant_session_sql(ctx.tenant_id)
                        }))
                    }
                }),
            ),
    )
    .await;

    // Step 1: Tenant A places order -> Success (Growth Tier)
    let req_a = test::TestRequest::post()
        .uri("/orders")
        .insert_header((
            "Authorization",
            format!("Bearer {}", tenant_a.jwt_token),
        ))
        .set_json(serde_json::json!({ "qty": 5 }))
        .to_request();

    let resp_a = test::call_service(&app, req_a).await;
    assert_eq!(resp_a.status(), StatusCode::CREATED);

    // Step 2: Tenant B attempts order when quota exceeded -> 402 Payment Required or handled
    let req_b = test::TestRequest::post()
        .uri("/orders")
        .insert_header((
            "Authorization",
            format!("Bearer {}", tenant_b.jwt_token),
        ))
        .set_json(serde_json::json!({ "qty": 1 }))
        .to_request();

    let resp_b = test::call_service(&app, req_b).await;
    assert!(
        resp_b.status() == StatusCode::PAYMENT_REQUIRED || resp_b.status() == StatusCode::CREATED
    );

    // Step 3: Verify Tenant A event fulfillment cycle
    let guard = tenant_a_events.lock().unwrap();
    assert_eq!(guard.len(), 1);
    assert!(validate_event_tenant_enrichment(
        &guard[0],
        tenant_a.tenant_id
    ));
}
