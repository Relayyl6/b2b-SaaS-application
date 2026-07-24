use crate::models::OrderEvent;
use crate::redis_pub::RedisPublisher;
use sqlx::PgPool;

pub async fn update_order_failed_event(
    pool: &PgPool,
    _redis_pub: &RedisPublisher, // Not emitting another event for failed here
    event: OrderEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let order_id = event.order_id.ok_or("No order_id found")?;
    match crate::db::update_order_status_db(pool, order_id, crate::models::OrderStatus::Failed, None, None, None).await {
        Ok(_) => println!("🔁({}) Updated order {:?} via DB", event.event_type, order_id),
        Err(e) => eprintln!("❌ Failed to update order status: {:?}", e),
    }
    Ok(())
}

pub async fn update_order_confirmed_event(
    pool: &PgPool,
    _redis_pub: &RedisPublisher,
    event: OrderEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let order_id = event.order_id.ok_or("No order_id found")?;
    match crate::db::update_order_status_db(pool, order_id, crate::models::OrderStatus::Confirmed, None, None, None).await {
        Ok(_) => println!("🔁({}) Updated order {:?} via DB", event.event_type, order_id),
        Err(e) => eprintln!("❌ Failed to update order status: {:?}", e),
    }
    Ok(())
}

pub async fn update_order_cancelled_event(
    pool: &PgPool,
    redis_pub: &RedisPublisher,
    event: OrderEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let order_id = event.order_id.ok_or("No order_id found")?;
    match crate::db::update_order_status_db(pool, order_id, crate::models::OrderStatus::Cancelled, None, None, None).await {
        Ok(order) => {
            println!("🔁({}) Updated order {:?} via DB", event.event_type, order_id);
            let cancel_event = OrderEvent {
                event_type: "order.cancelled".to_string(),
                product_id: order.product_id,
                supplier_id: order.supplier_id,
                order_id: Some(order.id),
                quantity: order.qty,
                user_id: Some(order.user_id),
                timestamp: order.order_timestamp,
                ..Default::default()
            };
            redis_pub.publish_async("order.cancelled", cancel_event);
        },
        Err(e) => eprintln!("❌ Failed to update order status: {:?}", e),
    }
    Ok(())
}

pub async fn update_order_shipped_event(
    pool: &PgPool,
    redis_pub: &RedisPublisher,
    event: OrderEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let order_id = event.order_id.ok_or("No order_id found")?;
    match crate::db::update_order_status_db(pool, order_id, crate::models::OrderStatus::Shipped, None, None, None).await {
        Ok(order) => {
            println!("🔁({}) Updated order {:?} via DB", event.event_type, order_id);
            let shipped_event = OrderEvent {
                event_type: "order.shipped".to_string(),
                product_id: order.product_id,
                supplier_id: order.supplier_id,
                order_id: Some(order.id),
                quantity: order.qty,
                user_id: Some(order.user_id),
                timestamp: order.order_timestamp,
                ..Default::default()
            };
            redis_pub.publish_async("order.shipped", shipped_event);
        },
        Err(e) => eprintln!("❌ Failed to update order status: {:?}", e),
    }
    Ok(())
}

pub async fn update_order_delivered_event(
    pool: &PgPool,
    _redis_pub: &RedisPublisher,
    event: OrderEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let order_id = event.order_id.ok_or("No order_id found")?;
    match crate::db::update_order_status_db(pool, order_id, crate::models::OrderStatus::Delivered, None, None, None).await {
        Ok(_) => println!("🔁({}) Updated order {:?} via DB", event.event_type, order_id),
        Err(e) => eprintln!("❌ Failed to update order status: {:?}", e),
    }
    Ok(())
}

