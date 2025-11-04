use sqlx::PgPool;
use uuid::Uuid;
use reqwest::Client;

struct ProductEvent {
    event_type: String, // e.g. "product.created"
    product_id: Uuid,
    supplier_id: Uuid,
    name: String,
    quantity: Option<i32>,
    low_stock_threshold: Option<i32>,
    unit: Option<String>,
}

// Create a new product in inventory when the Product Catalog announces a new one
pub async fn create_product_from_event(pool: &PgPool, event: ProductEvent) -> Result<()> {
    let client = Client::new();
    let service_url = env::var("INVENTORY_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3002".into());

    let url = format!("{}/inventory", service_url);

    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "product_id": event.product_id,
            "supplier_id": event.supplier_id,
            "quantity": event.quantity.unwrap_or(0),
            "name": event.name,
            "low_stock_threshold": event.low_stock_threshold.unwrap_or(5),
            "unit": event.unit.unwrap_or("unit".to_string())
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("✅({}) Created product {} via API route", event.event_type, event.name);
    } else {
        eprintln!("❌ Failed to create product: {:?}", resp.text().await?);
    }

    Ok(())
}

// Update an existing product (like when product catalog changes its name, unit, etc.)
pub async fn update_product_from_event(pool: &PgPool, event: ProductEvent) -> Result<()> {
    let client = Client::new();
    let service_url = env::var("INVENTORY_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3002".into());

    let url = format!("{}/inventory/{}/update", service_url, event.supplier_id);

    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "product_id": event.product_id,
            "quantity_change": event.quantity_change.unwrap_or(0)
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("🔁({}) Updated product {} via API route", event.event_type, event.name);
    } else {
        eprintln!("❌ Failed to update product: {:?}", resp.text().await?);
    }

    Ok(())
}

// Delete a product when Product Catalog says it's deleted
pub async fn delete_product_from_event(pool: &PgPool, event: ProductEvent) -> Result<()> {
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
