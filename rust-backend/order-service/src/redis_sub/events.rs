use reqwest::Client;
use std::env;
use serde_json;
use sqlx::PgPool;
use crate::models::OrderEvent;

pub async fn update_order_failed_event(
    _pool: &PgPool,
    event: OrderEvent
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let service_url = env::var("ORDER_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3005".into());
    let url = match event.order_id {
        Some(id) => format!("{}/orders/{}/status", service_url, id),
        None => {
            eprintln!("No order_id found, cannot create URL");
            return Ok(()); // or skip this event
        }
    };


    let resp = client
        .put(&url)
        .json(&serde_json::json!({
            "id": event.order_id,
            "product_id": event.product_id,
            "user_id": event.user_id,
            "new_status": "failed".to_string(),
            "timestamp": event.timestamp,
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("🔁({}) Updated order {:?} via API route", event.event_type, event.order_id);
    } else {
        eprintln!("❌ Failed to update product: {:?}", resp.text().await?);
    }

    Ok(())
}

pub async fn update_order_confirmed_event(_pool: &PgPool, event: OrderEvent) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let service_url = env::var("ORDER_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3005".into());
    let url = match event.order_id {
        Some(id) => format!("{}/orders/{}/status", service_url, id),
        None => {
            eprintln!("No order_id found, cannot create URL");
            return Ok(()); // or skip this event
        }
    };


    let resp = client
        .put(&url)
        .json(&serde_json::json!({
            "id": event.order_id,
            "product_id": event.product_id,
            "user_id": event.user_id,
            "new_status": "confirmed".to_string(),
            "timestamp": event.timestamp,
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("🔁({}) Updated order {:?} via API route", event.event_type, event.order_id);
    } else {
        eprintln!("❌ Failed to update product: {:?}", resp.text().await?);
    }

    Ok(())
}

pub async fn update_order_cancelled_event(_pool: &PgPool, event: OrderEvent) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let service_url = env::var("ORDER_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3005".into());
    let url = match event.order_id {
        Some(id) => format!("{}/orders/{}/status", service_url, id),
        None => {
            eprintln!("No order_id found, cannot create URL");
            return Ok(()); // or skip this event
        }
    };


    let resp = client
        .put(&url)
        .json(&serde_json::json!({
            "id": event.order_id,
            "product_id": event.product_id,
            "user_id": event.user_id,
            "new_status": "cancelled".to_string(),
            "timestamp": event.timestamp,
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("🔁({}) Updated order {:?} via API route", event.event_type, event.order_id);
    } else {
        eprintln!("❌ Failed to update product: {:?}", resp.text().await?);
    }

    Ok(())
}

pub async fn update_order_shipped_event(_pool: &PgPool, event: OrderEvent) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let service_url = env::var("ORDER_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3005".into());
    let url = match event.order_id {
        Some(id) => format!("{}/orders/{}/status", service_url, id),
        None => {
            eprintln!("No order_id found, cannot create URL");
            return Ok(()); // or skip this event
        }
    };


    let resp = client
        .put(&url)
        .json(&serde_json::json!({
            "id": event.order_id,
            "product_id": event.product_id,
            "user_id": event.user_id,
            "new_status": "shipped".to_string(),
            "timestamp": event.timestamp,
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("🔁({}) Updated order {:?} via API route", event.event_type, event.order_id);
    } else {
        eprintln!("❌ Failed to update product: {:?}", resp.text().await?);
    }

    Ok(())
}

pub async fn update_order_delivered_event(_pool: &PgPool, event: OrderEvent) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let service_url = env::var("ORDER_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3005".into());
    let url = match event.order_id {
        Some(id) => format!("{}/orders/{}/status", service_url, id),
        None => {
            eprintln!("No order_id found, cannot create URL");
            return Ok(()); // or skip this event
        }
    };


    let resp = client
        .put(&url)
        .json(&serde_json::json!({
            "id": event.order_id,
            "product_id": event.product_id,
            "user_id": event.user_id,
            "new_status": "delivered".to_string(),
            "timestamp": event.timestamp,
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("🔁({}) Updated order {:?} via API route", event.event_type, event.order_id);
    } else {
        eprintln!("❌ Failed to update product: {:?}", resp.text().await?);
    }

    Ok(())
}


