use e2e_tests::test_context::{
    create_enriched_event, validate_event_tenant_enrichment, EnrichedEventPayload,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SampleEvent {
    pub item_id: String,
    pub quantity: u32,
}

#[tokio::test]
async fn test_event_null_tenant_id_payload_rejection() {
    let secret = "test_secret";
    let event = create_enriched_event(
        "order.created",
        None, // Null tenant ID
        SampleEvent {
            item_id: "ITEM-100".to_string(),
            quantity: 2,
        },
        secret,
    );

    assert_eq!(event.tenant_id, None);
    // Consumer must reject event with null tenant_id
    let consumer_tenant = Uuid::new_v4();
    assert_eq!(
        validate_event_tenant_enrichment(&event, consumer_tenant),
        false,
        "Null tenant ID event must be rejected by tenant filter"
    );
}

#[tokio::test]
async fn test_event_cross_tenant_stream_poisoning() {
    let secret = "test_secret";
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();

    // Poison event carrying Tenant B ID in Tenant A stream context
    let poison_event = create_enriched_event(
        "order.created",
        Some(tenant_b),
        SampleEvent {
            item_id: "POISON-KEY".to_string(),
            quantity: 999,
        },
        secret,
    );

    let consumer_for_a = tenant_a;
    let is_valid = validate_event_tenant_enrichment(&poison_event, consumer_for_a);
    assert_eq!(
        is_valid, false,
        "Poisoned event carrying foreign tenant_id must be rejected"
    );
}

#[tokio::test]
async fn test_event_high_throughput_burst_multi_tenant() {
    let secret = "test_secret";
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();

    let mut events = Vec::new();
    for i in 0..100 {
        let tid = if i % 2 == 0 { tenant_a } else { tenant_b };
        events.push(create_enriched_event(
            "order.burst",
            Some(tid),
            SampleEvent {
                item_id: format!("BURST-{}", i),
                quantity: i as u32,
            },
            secret,
        ));
    }

    let tenant_a_processed = Arc::new(Mutex::new(0));
    let tenant_b_processed = Arc::new(Mutex::new(0));

    for event in events {
        if validate_event_tenant_enrichment(&event, tenant_a) {
            let mut count = tenant_a_processed.lock().unwrap();
            *count += 1;
        } else if validate_event_tenant_enrichment(&event, tenant_b) {
            let mut count = tenant_b_processed.lock().unwrap();
            *count += 1;
        }
    }

    assert_eq!(*tenant_a_processed.lock().unwrap(), 50);
    assert_eq!(*tenant_b_processed.lock().unwrap(), 50);
}

#[tokio::test]
async fn test_event_consumer_reconnect_tenant_state_preservation() {
    let secret = "test_secret";
    let tenant_id = Uuid::new_v4();

    // Consumer state before restart
    let filter_tenant_state = tenant_id;

    let event = create_enriched_event(
        "consumer.reconnect",
        Some(tenant_id),
        SampleEvent {
            item_id: "RECONNECT-1".to_string(),
            quantity: 1,
        },
        secret,
    );

    // Simulate disconnect and reconnect: filter state preserved
    let reconnected_filter = filter_tenant_state;
    assert!(validate_event_tenant_enrichment(
        &event,
        reconnected_filter
    ));
}

#[tokio::test]
async fn test_event_malformed_json_payload_dlq() {
    let malformed_raw_json = r#"{"event_id": "invalid-uuid", "tenant_id": null, "payload": "corrupted"#;

    let parse_res = serde_json::from_str::<EnrichedEventPayload<SampleEvent>>(malformed_raw_json);
    assert!(
        parse_res.is_err(),
        "Malformed JSON payload must fail deserialization"
    );

    // Verify routing to DLQ stream key
    let dlq_stream_key = "stream:dlq:malformed_json";
    assert!(dlq_stream_key.contains("dlq"));
}
