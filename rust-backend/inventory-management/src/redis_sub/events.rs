use sqlx::PgPool;
use uuid::Uuid;
use reqwest::Client;
use std::env;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductEvent {
    pub event_type: String,
    pub product_id: Uuid,
    pub supplier_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub category: Option<String>,
    pub quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
    pub unit: Option<String>,
    pub quantity_change: Option<i32>,
}

pub async fn create_product_from_event(_pool: &PgPool, event: ProductEvent) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let service_url = env::var("INVENTORY_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3002".into());
    let url = format!("{}/inventory", service_url);

    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "product_id": event.product_id,
            "supplier_id": event.supplier_id,
            "quantity": event.quantity.unwrap_or(0),
            "name": event.name.clone().unwrap_or("Unnamed product".to_string()),
            "description": event.description.clone().unwrap_or("No description for this product".to_string()),
            "price": event.price.unwrap_or(0.00),
            "category": event.category.clone().unwrap_or("Unspecified Category".to_string()),
            "low_stock_threshold": event.low_stock_threshold.unwrap_or(5),
            "unit": event.unit.unwrap_or("unit".to_string())
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("✅({}) Created product {:?} via API route", event.event_type, event.name);
    } else {
        eprintln!("❌ Failed to create product: {:?}", resp.text().await?);
    }

    Ok(())
}

pub async fn update_product_from_event(_pool: &PgPool, event: ProductEvent) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let service_url = env::var("INVENTORY_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3002".into());
    let url = format!("{}/inventory/{}/update", service_url, event.supplier_id);

    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "product_id": event.product_id,
            "supplier_id": event.supplier_id,
            "name": event.name,
            "description": event.description,
            "price": event.price,
            "category": event.category,
            "unit": event.unit,
            "quantity": event.quantity,
            "quantity_change": event.quantity_change,
            // Add more fields if your Inventory Service expects them
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("🔁({}) Updated product {:?} via API route", event.event_type, event.name);
    } else {
        eprintln!("❌ Failed to update product: {:?}", resp.text().await?);
    }

    Ok(())
}

pub async fn delete_product_from_event(_pool: &PgPool, event: ProductEvent) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let service_url = env::var("INVENTORY_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3002".into());
    let url = format!("{}/inventory/{}/{}", service_url, event.supplier_id, event.product_id);

    let resp = client.delete(&url).send().await?;

    if resp.status().is_success() {
        println!("🗑️({}) Deleted product {} via API route", event.event_type, event.product_id);
    } else {
        eprintln!("❌ Failed to delete product: {:?}", resp.text().await?);
    }

    Ok(())
}
