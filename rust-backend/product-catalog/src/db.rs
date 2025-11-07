use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Product, CreateProductRequest, UpdateProductRequest};

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
        let quantity = req.quantity.unwrap_or(0);

        sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (product_id, supplier_id, name, description, category, price, unit, quantity, available)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, product_id, supplier_id, name, description, category, price, unit, quantity, available, created_at, updated_at
            "#
        )
        .bind(&req.product_id)
        .bind(req.supplier_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.category)
        .bind(req.price)
        .bind(&req.unit)
        .bind(quantity)
        .bind(available)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_by_supplier(&self, supplier_id: Uuid) -> Result<Vec<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            r#"
            SELECT id, product_id, supplier_id, name, description, category, price, unit, quantity, available, created_at, updated_at
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
            SELECT id, supplier_id, product_id, name, description, category, price, unit, quantity, available, created_at, updated_at
            FROM products
            WHERE supplier_id = $1 AND product_id = $2
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
        sqlx::query_as::<_, Product>(
            r#"
            UPDATE products
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
              updated_at = NOW()
            WHERE supplier_id = $9 AND product_id = $10
            RETURNING id, product_id, supplier_id, name, description, category, price, unit, quantity, available, created_at, updated_at
            "#
        )
        .bind(req.name.as_ref())
        .bind(req.description.as_ref())
        .bind(req.category.as_ref())
        .bind(req.price)
        .bind(req.unit.as_ref())
        .bind(req.quantity)
        .bind(req.available)
        .bind(req.quantity_change) // new addition
        .bind(supplier_id)
        .bind(product_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_product(&self, supplier_id: Uuid, product_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"DELETE FROM products WHERE supplier_id = $1 AND product_id = $2"#,
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
        product_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Product>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Product>(
            r#"
            SELECT id, product_id, supplier_id, name, description, category, price, unit, quantity, available, created_at, updated_at
            FROM products
            WHERE ($1::text IS NULL OR category = $1)
              AND ($2::double precision IS NULL OR price >= $2)
              AND ($3::double precision IS NULL OR price <= $3)
              AND ($4::uuid IS NULL OR supplier_id = $4)
              AND ($5::uuid IS NULL OR product_id = $5)
            ORDER BY name
            LIMIT $6 OFFSET $7
            "#
        )
        .bind(category)
        .bind(min_price)
        .bind(max_price)
        .bind(supplier_id)
        .bind(product_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn bulk_create(&self, items: &[CreateProductRequest]) -> Result<Vec<Product>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::with_capacity(items.len());

        for it in items {
            let p = sqlx::query_as::<_, Product>(
                r#"
                INSERT INTO products (product_id, supplier_id, name, description, category, price, unit, quantity, available)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id, product_id, supplier_id, name, description, category, price, unit, quantity, available, created_at, updated_at
                "#
            )
            .bind(&it.product_id)
            .bind(it.supplier_id)
            .bind(&it.name)
            .bind(&it.description)
            .bind(&it.category)
            .bind(it.price)
            .bind(&it.unit)
            .bind(it.quantity.unwrap_or(0))
            .bind(it.available.unwrap_or(true))
            .fetch_one(&mut *tx)
            .await?;

            created.push(p);
        }

        tx.commit().await?;
        Ok(created)
    }
}
