#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_order_event_extracts_order_id() {
        let expected_id = Uuid::new_v4();

        let ev = AnalyticsEvent {
            event_type: "order.created".to_string(),
            order_id: Some(expected_id),
            product_id: None,
            supplier_id: None,
            user_id: None,
            name: None,
            description: None,
            price: None,
            category: None,
            low_stock_threshold: None,
            unit: None,
            available: None,
            quantity_change: None,
            quantity: None,
            reservation_id: None,
            timestamp: Some(Utc::now()),
            expires_at: None,
        };

        assert_eq!(ev.extract_primary_id(), expected_id);
    }

    #[test]
    fn test_product_event_extracts_product_id() {
        let expected_id = Uuid::new_v4();

        let ev = AnalyticsEvent {
            event_type: "product.updated".to_string(),
            product_id: Some(expected_id),
            order_id: None,
            supplier_id: None,
            user_id: None,
            name: None,
            description: None,
            price: None,
            category: None,
            low_stock_threshold: None,
            unit: None,
            available: None,
            quantity_change: None,
            quantity: None,
            reservation_id: None,
            timestamp: None,
            expires_at: None,
        };

        assert_eq!(ev.extract_primary_id(), expected_id);
    }

    #[test]
    fn test_unknown_event_generates_new_id() {
        let ev = AnalyticsEvent {
            event_type: "random.event".to_string(),
            order_id: None,
            product_id: None,
            supplier_id: None,
            user_id: None,
            name: None,
            description: None,
            price: None,
            category: None,
            low_stock_threshold: None,
            unit: None,
            available: None,
            quantity_change: None,
            quantity: None,
            reservation_id: None,
            timestamp: None,
            expires_at: None,
        };

        // Should NOT panic
        let id = ev.extract_primary_id();

        // ID must be valid
        assert_ne!(id, Uuid::nil());
    }

    #[test]
    fn test_event_new_serializes_data() {
        let ev = AnalyticsEvent {
            event_type: "user.logged_in".to_string(),
            user_id: Some(Uuid::new_v4()),
            order_id: None,
            product_id: None,
            supplier_id: None,
            name: None,
            description: None,
            price: None,
            category: None,
            low_stock_threshold: None,
            unit: None,
            available: None,
            quantity_change: None,
            quantity: None,
            reservation_id: None,
            timestamp: Some(Utc::now()),
            expires_at: None,
        };

        let event = Event::new(ev).expect("failed to create Event");
        assert!(event.data.is_object());
        assert!(event.data.get("event_type").is_some());
    }
}
