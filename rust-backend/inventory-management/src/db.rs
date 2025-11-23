// src/db.rs
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Inventory, UpdateStockRequest};

#[derive(Clone)]
pub struct InventoryRepo {
    pool: PgPool,
}

impl InventoryRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
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
            SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                category = COALESCE($3, category),
                price = COALESCE($4, price),
                unit = COALESCE($5, unit),
                quantity = COALESCE(
                    CASE
                        WHEN $8 IS NOT NULL THEN quantity + $8
                        ELSE $6
                    END,
                    quantity
                ),
                available = COALESCE($7, available),
                low_stock_threshold = COALESCE($9, low_stock_threshold),
                updated_at = NOW()
            WHERE supplier_id = $10 AND product_id = $11
            RETURNING *
            "#
        )
        .bind(req.name.as_ref())
        .bind(req.description.as_ref())
        .bind(req.category.as_ref())
        .bind(req.price)
        .bind(req.unit.as_ref())
        .bind(req.quantity)
        .bind(req.available)
        .bind(req.quantity_change)
        .bind(req.low_stock_threshold)
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
            INSERT INTO inventory (supplier_id, product_id, name, quantity, low_stock_threshold, unit, description, price, category)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(req.supplier_id)
        .bind(req.product_id)
        .bind(&req.name)
        .bind(req.quantity)
        .bind(req.low_stock_threshold)
        .bind(&req.unit)
        .bind(&req.description)
        .bind(req.price)
        .bind(&req.category)
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