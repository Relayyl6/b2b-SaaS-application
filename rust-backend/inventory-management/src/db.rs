// src/db.rs
use sqlx::PgPool;
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
        sqlx::query_as::<_, Inventory>(
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
        .await
    }

    pub async fn create_inventory_item(
        &self,
        req: &crate::models::CreateInventoryRequest,
    ) -> Result<Inventory, sqlx::Error> {
        sqlx::query_as::<_, Inventory>(
            r#"
            INSERT INTO inventory (supplier_id, product_id, name, quantity, low_stock_threshold, unit)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#
        )
        .bind(req.supplier_id)
        .bind(req.product_id)
        .bind(&req.name)
        .bind(req.quantity)
        .bind(req.low_stock_threshold)
        .bind(&req.unit)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_one(
        &self,
        supplier_id: Uuid,
        product_id: Uuid,
    ) -> Result<Inventory, sqlx::Error> {
        sqlx::query_as::<_, Inventory>(
            r#"
            SELECT * FROM inventory
            WHERE supplier_id = $1 AND product_id = $2
            "#,
        )
        .bind(supplier_id)
        .bind(product_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_product(
        &self,
        supplier_id: Uuid,
        product_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM inventory WHERE supplier_id = $1 AND product_id = $2",
            supplier_id,
            product_id
        )
        .execute(&self.pool)
        .await?;
    
        Ok(result.rows_affected())
    }

}