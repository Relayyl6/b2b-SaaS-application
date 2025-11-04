use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Product, CreateProductRequest, UpdateProductRequest};
use chrono::{Utc, DateTime};

#[derive(Clone)]
pub struct ProductRepo {
    pool: PgPool,
}

impl ProductRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_product(&self, req: &CreateProductRequest) -> Result<Product, sqlx::Error> {
        let available = req.available.unwrap_or(true);
        // Note: explicit column list + RETURNING to avoid "SELECT *" issues
        sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (supplier_id, name, description, category, price, unit, available)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, supplier_id, name, description, category, price, unit, available, created_at, updated_at
            "#
        )
        .bind(req.supplier_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.category)
        .bind(req.price)
        .bind(&req.unit)
        .bind(available)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_by_supplier(&self, supplier_id: Uuid) -> Result<Vec<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            r#"
            SELECT id, supplier_id, name, description, category, price, unit, available, created_at, updated_at
            FROM products
            WHERE supplier_id = $1
            ORDER BY name
            "#
        )
        .bind(supplier_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_one(&self, supplier_id: Uuid, product_id: Uuid) -> Result<Product, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            r#"
            SELECT id, supplier_id, name, description, category, price, unit, available, created_at, updated_at
            FROM products
            WHERE supplier_id = $1 AND id = $2
            "#
        )
        .bind(supplier_id)
        .bind(product_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_product(
        &self,
        supplier_id: Uuid,
        product_id: Uuid,
        req: &UpdateProductRequest,
    ) -> Result<Product, sqlx::Error> {
        // simpler approach: build UPDATE using COALESCE-like style; we set value = COALESCE($1, column)
        sqlx::query_as::<_, Product>(
            r#"
            UPDATE products
            SET
              name = COALESCE($1, name),
              description = COALESCE($2, description),
              category = COALESCE($3, category),
              price = COALESCE($4, price),
              unit = COALESCE($5, unit),
              available = COALESCE($6, available),
              updated_at = NOW()
            WHERE supplier_id = $7 AND id = $8
            RETURNING id, supplier_id, name, description, category, price, unit, available, created_at, updated_at
            "#
        )
        .bind(req.name.as_ref())
        .bind(req.description.as_ref())
        .bind(req.category.as_ref())
        .bind(req.price)
        .bind(req.unit.as_ref())
        .bind(req.available)
        .bind(supplier_id)
        .bind(product_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_product(&self, supplier_id: Uuid, product_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"DELETE FROM products WHERE supplier_id = $1 AND id = $2"#,
            supplier_id,
            product_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn search_products(
        &self,
        category: Option<String>,
        min_price: Option<f64>,
        max_price: Option<f64>,
        supplier_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Product>, sqlx::Error> {
        // Build dynamic SQL lightly — for a more complete approach, use query_builder or write multiple queries
        let mut q = String::from("SELECT id, supplier_id, name, description, category, price, unit, available, created_at, updated_at FROM products WHERE 1=1");
        if category.is_some() { q.push_str(" AND category = $1"); }
        // For simplicity here we only show an example — production: use query builder or properly numbered binds
        // To keep compile-time safe, implement a few common combinations instead. For time's sake, use a simple example below:
        let rows = sqlx::query_as::<_, Product>(
            r#"
            SELECT id, supplier_id, name, description, category, price, unit, available, created_at, updated_at
            FROM products
            WHERE ($1::text IS NULL OR category = $1)
              AND ($2::double precision IS NULL OR price >= $2)
              AND ($3::double precision IS NULL OR price <= $3)
              AND ($4::uuid IS NULL OR supplier_id = $4)
            ORDER BY name
            LIMIT $5 OFFSET $6
            "#
        )
        .bind(category)
        .bind(min_price)
        .bind(max_price)
        .bind(supplier_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn bulk_create(&self, items: &[CreateProductRequest]) -> Result<Vec<Product>, sqlx::Error> {
        // naive bulk insert in a transaction
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::with_capacity(items.len());
        for it in items {
            let p = sqlx::query_as::<_, Product>(
                r#"
                INSERT INTO products (supplier_id, name, description, category, price, unit, available)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, supplier_id, name, description, category, price, unit, available, created_at, updated_at
                "#
            )
            .bind(it.supplier_id)
            .bind(&it.name)
            .bind(&it.description)
            .bind(&it.category)
            .bind(it.price)
            .bind(&it.unit)
            .bind(it.available.unwrap_or(true))
            .fetch_one(&mut *tx)
            .await?;

            created.push(p);
        }
        tx.commit().await?;
        Ok(created)
    }
}
