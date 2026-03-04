use crate::models::{CreateShipmentRequest, Shipment, ShipmentStatus, UpdateShipmentStatusRequest};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct LogisticsRepo {
    pool: PgPool,
}

impl LogisticsRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn create_shipment(
        &self,
        req: &CreateShipmentRequest,
    ) -> Result<Shipment, sqlx::Error> {
        let tracking_number = format!("TRK-{}", Uuid::new_v4().simple());

        sqlx::query_as::<_, Shipment>(
            r#"
            INSERT INTO shipments (id, order_id, user_id, supplier_id, product_id, tracking_number, status, notes)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(req.order_id)
        .bind(req.user_id)
        .bind(req.supplier_id)
        .bind(req.product_id)
        .bind(tracking_number)
        .bind(&req.notes)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_shipment(&self, shipment_id: Uuid) -> Result<Shipment, sqlx::Error> {
        sqlx::query_as::<_, Shipment>("SELECT * FROM shipments WHERE id = $1")
            .bind(shipment_id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_by_order_id(&self, order_id: Uuid) -> Result<Shipment, sqlx::Error> {
        sqlx::query_as::<_, Shipment>("SELECT * FROM shipments WHERE order_id = $1")
            .bind(order_id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn list_supplier_shipments(
        &self,
        supplier_id: Uuid,
    ) -> Result<Vec<Shipment>, sqlx::Error> {
        sqlx::query_as::<_, Shipment>(
            "SELECT * FROM shipments WHERE supplier_id = $1 ORDER BY created_at DESC",
        )
        .bind(supplier_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_status(
        &self,
        shipment_id: Uuid,
        req: &UpdateShipmentStatusRequest,
    ) -> Result<Shipment, sqlx::Error> {
        let dispatched_at = if req.status == ShipmentStatus::Intransit {
            Some(Utc::now())
        } else {
            None
        };

        let delivered_at = if req.status == ShipmentStatus::Delivered {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query_as::<_, Shipment>(
            r#"
            UPDATE shipments
            SET
                status = $1,
                notes = COALESCE($2, notes),
                dispatched_at = COALESCE($3, dispatched_at),
                delivered_at = COALESCE($4, delivered_at),
                updated_at = NOW()
            WHERE id = $5
            RETURNING *
            "#,
        )
        .bind(&req.status)
        .bind(&req.notes)
        .bind(dispatched_at)
        .bind(delivered_at)
        .bind(shipment_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn cancel_by_order_id(&self, order_id: Uuid) -> Result<Shipment, sqlx::Error> {
        sqlx::query_as::<_, Shipment>(
            r#"
            UPDATE shipments
            SET status = 'cancelled', updated_at = NOW()
            WHERE order_id = $1
            RETURNING *
            "#,
        )
        .bind(order_id)
        .fetch_one(&self.pool)
        .await
    }
}
