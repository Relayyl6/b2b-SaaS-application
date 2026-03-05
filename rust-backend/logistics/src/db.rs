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
    /// Update a shipment's status, setting dispatched_at when transitioning to `Intransit`
    /// and delivered_at when transitioning to `Delivered`, and return the updated shipment.
    ///
    /// # Errors
    ///
    /// Returns an `sqlx::Error::Protocol` if the requested status transition is not allowed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uuid::Uuid;
    /// # use chrono::Utc;
    /// # use logistics::db::LogisticsRepo;
    /// # use logistics::models::{UpdateShipmentStatusRequest, ShipmentStatus};
    /// # async fn _example(repo: &LogisticsRepo) -> Result<(), sqlx::Error> {
    /// let req = UpdateShipmentStatusRequest {
    ///     status: ShipmentStatus::Intransit,
    ///     notes: Some("Out for delivery".into()),
    /// };
    /// let updated = repo.update_status(Uuid::nil(), &req).await?;
    /// assert_eq!(updated.status, ShipmentStatus::Intransit);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_status(
        &self,
        shipment_id: Uuid,
        req: &UpdateShipmentStatusRequest,
    ) -> Result<Shipment, sqlx::Error> {
        let current = self.get_shipment(shipment_id).await?;
        if !current.status.can_transition_to(&req.status) {
            return Err(sqlx::Error::Protocol(
                format!(
                    "invalid status transition: {:?} -> {:?}",
                    current.status, req.status
                )
                .into(),
            ));
        }

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

    /// Cancels the shipment for an order when cancellation is allowed.
    /// Cancels the shipment for the given order if the shipment has not been delivered.
    ///
    /// If the shipment's current status is `delivered`, no update occurs and the query will not return a row.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use uuid::Uuid;
    /// # async fn run(repo: &crate::db::LogisticsRepo) -> Result<(), sqlx::Error> {
    /// let order_id = Uuid::new_v4();
    /// let shipment = repo.cancel_by_order_id(order_id).await?;
    /// assert_eq!(shipment.order_id, order_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cancel_by_order_id(&self, order_id: Uuid) -> Result<Shipment, sqlx::Error> {
        sqlx::query_as::<_, Shipment>(
            r#"
            UPDATE shipments
            SET status = 'cancelled', updated_at = NOW()
            WHERE order_id = $1
              AND status != 'delivered'
            RETURNING *
            "#,
        )
        .bind(order_id)
        .fetch_one(&self.pool)
        .await
    }
}
