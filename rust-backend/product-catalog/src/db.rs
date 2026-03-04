use crate::models::{
    CreateProductRequest, Product, ProductAsset, RegisterProductAssetRequest, UpdateProductRequest,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ProductRepo {
    pool: PgPool,
}

impl ProductRepo {
    /// Creates a new instance with the provided dependencies.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a product and emits best-effort integration events.
    pub async fn create_product(&self, req: &CreateProductRequest) -> Result<Product, sqlx::Error> {
        let available = req.available.unwrap_or(true);
        let quantity = req.quantity.unwrap_or(0);
        let product_id = req.product_id.unwrap_or_else(Uuid::new_v4);
        let low_stock_threshold = req.low_stock_threshold.unwrap_or(10);

        sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (product_id, supplier_id, name, description, category, price, unit, quantity, available, low_stock_threshold)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, product_id, supplier_id, name, description, category, price, unit, quantity, available, low_stock_threshold, created_at, updated_at
            "#,
        )
        .bind(product_id)
        .bind(req.supplier_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.category)
        .bind(req.price)
        .bind(&req.unit)
        .bind(quantity)
        .bind(available)
        .bind(low_stock_threshold)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_by_supplier(&self, supplier_id: Uuid) -> Result<Vec<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            r#"
                SELECT id, product_id, supplier_id, name, description, category, price, unit, quantity, available, low_stock_threshold, created_at, updated_at
                FROM products
                WHERE supplier_id = $1
                ORDER BY name
            "#,
        )
        .bind(supplier_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Returns a single product for a supplier/product pair.
    pub async fn get_one(
        &self,
        supplier_id: Uuid,
        product_id: Uuid,
    ) -> Result<Product, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            r#"
            SELECT id, supplier_id, product_id, name, description, category, price, unit, quantity, available, low_stock_threshold, created_at, updated_at
            FROM products
            WHERE supplier_id = $1 AND product_id = $2
            "#,
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
              low_stock_threshold = COALESCE($9, low_stock_threshold),
              updated_at = NOW()
            WHERE supplier_id = $10 AND product_id = $11
            RETURNING id, product_id, supplier_id, name, description, category, price, unit, quantity, available, low_stock_threshold, created_at, updated_at
            "#,
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
        .bind(product_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_product(
        &self,
        supplier_id: Uuid,
        product_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result =
            sqlx::query(r#"DELETE FROM products WHERE supplier_id = $1 AND product_id = $2"#)
                .bind(supplier_id)
                .bind(product_id)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected())
    }

    /// Searches products by optional query parameters.
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
            SELECT id, product_id, supplier_id, name, description, category, price, unit, quantity, available, low_stock_threshold, created_at, updated_at
            FROM products
            WHERE ($1::text IS NULL OR category = $1)
              AND ($2::double precision IS NULL OR price >= $2)
              AND ($3::double precision IS NULL OR price <= $3)
              AND ($4::uuid IS NULL OR supplier_id = $4)
              AND ($5::uuid IS NULL OR product_id = $5)
            ORDER BY name
            LIMIT $6 OFFSET $7
            "#,
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

    pub async fn bulk_create(
        &self,
        items: &[CreateProductRequest],
    ) -> Result<Vec<Product>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::with_capacity(items.len());

        for it in items {
            let p = sqlx::query_as::<_, Product>(
                r#"
                INSERT INTO products (product_id, supplier_id, name, description, category, price, unit, quantity, available, low_stock_threshold)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING id, product_id, supplier_id, name, description, category, price, unit, quantity, available, low_stock_threshold, created_at, updated_at
                "#,
            )
            .bind(it.product_id.unwrap_or_else(Uuid::new_v4))
            .bind(it.supplier_id)
            .bind(&it.name)
            .bind(&it.description)
            .bind(&it.category)
            .bind(it.price)
            .bind(&it.unit)
            .bind(it.quantity.unwrap_or(0))
            .bind(it.available.unwrap_or(true))
            .bind(it.low_stock_threshold.unwrap_or(10))
            .fetch_one(&mut *tx)
            .await?;

            created.push(p);
        }

        tx.commit().await?;
        Ok(created)
    }

    pub async fn register_product_asset(
        &self,
        supplier_id: Uuid,
        product_id: Uuid,
        req: &RegisterProductAssetRequest,
    ) -> Result<ProductAsset, sqlx::Error> {
        let is_primary = req.is_primary.unwrap_or(false);

        let mut tx = self.pool.begin().await?;
        let exists = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT 1
            FROM products
            WHERE supplier_id = $1 AND product_id = $2
            "#,
        )
        .bind(supplier_id)
        .bind(product_id)
        .fetch_optional(&mut *tx)
        .await?;

        if exists.is_none() {
            return Err(sqlx::Error::RowNotFound);
        }
        if is_primary {
            sqlx::query(
                r#"
                UPDATE product_assets
                SET is_primary = FALSE
                WHERE supplier_id = $1 AND product_id = $2
                "#,
            )
            .bind(supplier_id)
            .bind(product_id)
            .execute(&mut *tx)
            .await?;
        }

        let asset = sqlx::query_as::<_, ProductAsset>(
            r#"
            INSERT INTO product_assets (
                id, product_id, supplier_id, provider, public_id, url, secure_url,
                width, height, bytes, format, alt_text, is_primary
            )
            VALUES (
                gen_random_uuid(), $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11, $12
            )
            RETURNING id, product_id, supplier_id, provider, public_id, url, secure_url,
                      width, height, bytes, format, alt_text, is_primary, created_at
            "#,
        )
        .bind(product_id)
        .bind(supplier_id)
        .bind(req.provider.as_deref().unwrap_or("cloudinary"))
        .bind(&req.public_id)
        .bind(&req.url)
        .bind(&req.secure_url)
        .bind(req.width)
        .bind(req.height)
        .bind(req.bytes)
        .bind(req.format.as_deref())
        .bind(req.alt_text.as_deref())
        .bind(is_primary)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(asset)
    }

    pub async fn list_product_assets(
        &self,
        supplier_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<ProductAsset>, sqlx::Error> {
        sqlx::query_as::<_, ProductAsset>(
            r#"
            SELECT id, product_id, supplier_id, provider, public_id, url, secure_url,
                   width, height, bytes, format, alt_text, is_primary, created_at
            FROM product_assets
            WHERE supplier_id = $1 AND product_id = $2
            ORDER BY is_primary DESC, created_at DESC
            "#,
        )
        .bind(supplier_id)
        .bind(product_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn delete_product_asset(
        &self,
        supplier_id: Uuid,
        product_id: Uuid,
        asset_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM product_assets
            WHERE supplier_id = $1 AND product_id = $2 AND id = $3
            "#,
        )
        .bind(supplier_id)
        .bind(product_id)
        .bind(asset_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
