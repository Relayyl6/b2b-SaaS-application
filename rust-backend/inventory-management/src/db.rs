use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use crate::models::{Inventory, UpdateStockRequest};

#[derive(Clone)]
pub struct InventoryRepo {
    pool: PgPool,
}

impl InventoryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_supplier(&self, supplier_id: Uuid) -> Result<Vec<Inventory>, sqlx::Error> {
        sqlx::query_as::<_, Inventory>(
            "SELECT * FROM inventory WHERE supplier_id = $1 ORDER BY name"
        )
        .bind(supplier_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_stock(
        &self,
        supplier_id: Uuid,
        req: &UpdateStockRequest,
    ) -> Result<Inventory, sqlx::Error> {
        let updated = sqlx::query_as::<_, Inventory>(
            r#"
            UPDATE inventory
            SET quantity = GREATEST(0, quantity + $1), updated_at = NOW()
            WHERE supplier_id = $2 AND product_id = $3
            RETURNING *
            "#
        )
        .bind(req.quantity_change)
        .bind(supplier_id)
        .bind(req.product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }
}