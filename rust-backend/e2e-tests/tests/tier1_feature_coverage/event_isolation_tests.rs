use e2e_tests::test_context::{
    create_enriched_event, validate_event_tenant_enrichment, EnrichedEventPayload,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct OrderCreatedPayload {
    pub order_id: Uuid,
    pub amount: u64,
}

#[tokio::test]
async fn test_event_order_created_contains_tenant_id_enrichment() {
    let secret = "test_secret";
    let tenant_id = Uuid::new_v4();
    let order_id = Uuid::new_v4();

    let event = create_enriched_event(
        "order.created",
        Some(tenant_id),
        OrderCreatedPayload {
            order_id,
            amount: 150,
        },
        secret,
    );

    assert_eq!(event.event_type, "order.created");
    assert_eq!(event.tenant_id, Some(tenant_id));
    assert!(validate_event_tenant_enrichment(&event, tenant_id));
    assert!(event.signature.is_some());
}

#[tokio::test]
async fn test_event_consumer_processes_matching_tenant_event() {
    let secret = "test_secret";
    let tenant_a = Uuid::new_v4();
    let event = create_enriched_event(
        "order.created",
        Some(tenant_a),
        OrderCreatedPayload {
            order_id: Uuid::new_v4(),
            amount: 100,
        },
        secret,
    );

    let processed = Arc::new(Mutex::new(false));
    let consumer_tenant_id = tenant_a;

    let handler = |e: EnrichedEventPayload<OrderCreatedPayload>| {
        if validate_event_tenant_enrichment(&e, consumer_tenant_id) {
            Ok::<_, String>(true)
        } else {
            Err("Mismatched tenant".to_string())
        }
    };

    let result = handler(event);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn test_event_consumer_rejects_mismatched_tenant_event() {
    let secret = "test_secret";
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();

    // Event carries Tenant B
    let event = create_enriched_event(
        "order.created",
        Some(tenant_b),
        OrderCreatedPayload {
            order_id: Uuid::new_v4(),
            amount: 200,
        },
        secret,
    );

    // Consumer expects Tenant A
    let consumer_tenant_id = tenant_a;
    let handler = |e: EnrichedEventPayload<OrderCreatedPayload>| {
        if validate_event_tenant_enrichment(&e, consumer_tenant_id) {
            Ok::<_, String>(true)
        } else {
            Err("TENANT_MISMATCH".to_string())
        }
    };

    let result = handler(event);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "TENANT_MISMATCH");
}

#[tokio::test]
async fn test_event_envelope_metadata_carries_tenant_context() {
    let secret = "test_secret";
    let tenant_id = Uuid::new_v4();
    let event = create_enriched_event(
        "inventory.reserved",
        Some(tenant_id),
        serde_json::json!({ "sku": "PROD-123", "qty": 5 }),
        secret,
    );

    let json_str = serde_json::to_string(&event).unwrap();
    assert!(json_str.contains("tenant_id"));
    assert!(json_str.contains(&tenant_id.to_string()));
}

#[tokio::test]
async fn test_event_mismatched_tenant_event_routed_to_dlq() {
    let secret = "test_secret";
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();

    let event = create_enriched_event(
        "payment.processed",
        Some(tenant_b),
        serde_json::json!({ "payment_id": Uuid::new_v4() }),
        secret,
    );

    let dlq_events: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let dlq_clone = dlq_events.clone();

    let process_event = move |e: EnrichedEventPayload<serde_json::Value>| {
        if !validate_event_tenant_enrichment(&e, tenant_a) {
            let mut guard = dlq_clone.lock().unwrap();
            guard.push(format!("stream:dlq:tenant_mismatch:{}", e.event_id));
            return Err("Routed to DLQ");
        }
        Ok("Success")
    };

    let res = process_event(event.clone());
    assert!(res.is_err());

    let guard = dlq_events.lock().unwrap();
    assert_eq!(guard.len(), 1);
    assert!(guard[0].contains("stream:dlq:tenant_mismatch"));
}
