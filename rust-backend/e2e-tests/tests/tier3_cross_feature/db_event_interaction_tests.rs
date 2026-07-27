use e2e_tests::test_context::{
    create_enriched_event, format_set_tenant_session_sql, validate_event_tenant_enrichment,
    EnrichedEventPayload,
};
use e2e_tests::TestHarness;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OrderEventData {
    pub order_id: Uuid,
    pub qty: i32,
}

#[tokio::test]
async fn test_cross_event_consumer_db_session_scoping() {
    let secret = "test_secret";
    let tenant_id = Uuid::new_v4();
    let order_id = Uuid::new_v4();

    let event = create_enriched_event(
        "order.created",
        Some(tenant_id),
        OrderEventData { order_id, qty: 5 },
        secret,
    );

    let harness = TestHarness::new().await;
    let executed_session_sql = Arc::new(Mutex::new(None));
    let sql_clone = executed_session_sql.clone();

    // Consumer receives event, extracts tenant_id, and sets DB session
    let consumer = move |e: EnrichedEventPayload<OrderEventData>| {
        if let Some(tid) = e.tenant_id {
            let sql = format_set_tenant_session_sql(tid);
            let mut guard = sql_clone.lock().unwrap();
            *guard = Some(sql);
            Ok::<_, String>("Scoped DB session and processed order")
        } else {
            Err("No tenant ID".to_string())
        }
    };

    let res = consumer(event);
    assert!(res.is_ok());

    let guard = executed_session_sql.lock().unwrap();
    assert!(guard.is_some());
    assert!(guard.as_ref().unwrap().contains(&tenant_id.to_string()));
}

#[tokio::test]
async fn test_cross_event_mismatch_prevents_db_write() {
    let secret = "test_secret";
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();

    let event = create_enriched_event(
        "order.created",
        Some(tenant_b), // Event belongs to Tenant B
        OrderEventData {
            order_id: Uuid::new_v4(),
            qty: 10,
        },
        secret,
    );

    let db_writes = Arc::new(Mutex::new(0));
    let writes_clone = db_writes.clone();

    let consumer_for_tenant_a = move |e: EnrichedEventPayload<OrderEventData>| {
        if !validate_event_tenant_enrichment(&e, tenant_a) {
            // Drop event; ZERO DB writes executed
            return Err("TENANT_MISMATCH_DROPPED");
        }

        let mut guard = writes_clone.lock().unwrap();
        *guard += 1;
        Ok("DB write completed")
    };

    let res = consumer_for_tenant_a(event);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "TENANT_MISMATCH_DROPPED");

    assert_eq!(
        *db_writes.lock().unwrap(),
        0,
        "Mismatched event must result in ZERO database writes"
    );
}
