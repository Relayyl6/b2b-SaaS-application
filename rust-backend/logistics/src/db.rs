use crate::models::{
    CreateShipmentRequest, ListShipmentQuery, Shipment, ShipmentStatus, UpdateShipmentStatusRequest,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct LogisticsRepo {
    pool: PgPool,
}

impl LogisticsRepo {
    /// Creates a new instance with the provided dependencies.
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    /// Creates a shipment and publishes logistics.shipment_created.
    pub async fn create_shipment(
        &self,
        req: &CreateShipmentRequest,
    ) -> Result<Shipment, sqlx::Error> {
        let tracking_number = format!("TRK-{}", Uuid::new_v4().simple());

        sqlx::query_as::<_, Shipment>(
            r#"
            INSERT INTO shipments (id, order_id, user_id, supplier_id, product_id, tracking_number, status, notes)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7)
            ON CONFLICT(order_id) DO UPDATE SET
                notes = COALESCE(EXCLUDED.notes, shipments.notes),
                updated_at = NOW()
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

    /// Returns shipment details by id.
    pub async fn get_shipment(&self, shipment_id: Uuid) -> Result<Shipment, sqlx::Error> {
        sqlx::query_as::<_, Shipment>("SELECT * FROM shipments WHERE id = $1")
            .bind(shipment_id)
            .fetch_one(&self.pool)
            .await
    }

    /// Returns one shipment by order id.
    pub async fn get_by_order_id(&self, order_id: Uuid) -> Result<Shipment, sqlx::Error> {
        sqlx::query_as::<_, Shipment>("SELECT * FROM shipments WHERE order_id = $1")
            .bind(order_id)
            .fetch_one(&self.pool)
            .await
    }

    /// Returns supplier shipments using filter and pagination query fields.
    pub async fn list_supplier_shipments(
        &self,
        supplier_id: Uuid,
        query: &ListShipmentQuery,
    ) -> Result<Vec<Shipment>, sqlx::Error> {
        let limit = query.limit.unwrap_or(50).clamp(1, 200);
        let offset = query.offset.unwrap_or(0).max(0);

        sqlx::query_as::<_, Shipment>(
            r#"
            SELECT *
            FROM shipments
            WHERE supplier_id = $1
              AND ($2::shipment_status IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(supplier_id)
        .bind(query.status.as_ref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    /// Updates shipment status and publishes logistics.shipment_updated.

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

        let res = sqlx::query_as::<_, Shipment>(
            r#"
            UPDATE shipments
            SET
                status = $1,
                notes = COALESCE($2, notes),
                dispatched_at = COALESCE($3, dispatched_at),
                delivered_at = COALESCE($4, delivered_at),
                updated_at = NOW()
            WHERE id = $5 AND (
                ($1 = 'intransit' AND status = 'pending') OR
                ($1 = 'delivered' AND status = 'intransit') OR
                ($1 = 'cancelled' AND status IN ('pending', 'intransit')) OR
                ($1 = status)
            )
            RETURNING *
            "#,
        )
        .bind(&req.status)
        .bind(&req.notes)
        .bind(dispatched_at)
        .bind(delivered_at)
        .bind(shipment_id)
        .fetch_optional(&self.pool)
        .await?;

        match res {
            Some(shipment) => Ok(shipment),
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    /// Cancels the shipment for an order when cancellation is allowed.
    /// Cancels the shipment for the given order if the shipment has not been delivered.
    pub async fn cancel_by_order_id(&self, order_id: Uuid) -> Result<Shipment, sqlx::Error> {
        sqlx::query_as::<_, Shipment>(
            r#"
            UPDATE shipments
            SET status = 'cancelled', updated_at = NOW()
            WHERE order_id = $1
              AND status IN ('pending', 'intransit')
            RETURNING *
            "#,
        )
        .bind(order_id)
        .fetch_one(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateShipmentRequest, ShipmentStatus, UpdateShipmentStatusRequest};
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_state_transitions_in_db(pool: sqlx::PgPool) {
        let repo = LogisticsRepo::new(&pool);
        let order_id = Uuid::new_v4();
        
        let req = CreateShipmentRequest {
            order_id,
            user_id: Uuid::new_v4(),
            supplier_id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            notes: None,
        };

        // 1. Create a shipment
        let shipment = repo.create_shipment(&req).await.unwrap();
        assert_eq!(shipment.status, ShipmentStatus::Pending);

        // 2. Invalid transition (Pending -> Delivered)
        let invalid_update = UpdateShipmentStatusRequest {
            status: ShipmentStatus::Delivered,
            notes: None,
        };
        let result = repo.update_status(shipment.id, &invalid_update).await;
        
        // Because of the WHERE clause in update_status, 0 rows are updated, returning RowNotFound.
        assert!(matches!(result, Err(sqlx::Error::RowNotFound)));

        // 3. Valid transition (Pending -> Intransit)
        let valid_update = UpdateShipmentStatusRequest {
            status: ShipmentStatus::Intransit,
            notes: None,
        };
        let updated = repo.update_status(shipment.id, &valid_update).await.unwrap();
        assert_eq!(updated.status, ShipmentStatus::Intransit);
    }
}
