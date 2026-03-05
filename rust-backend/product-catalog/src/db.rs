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

    /// Retrieve all products for the given supplier, ordered by name.
    ///
    /// The returned vector contains `Product` rows whose `supplier_id` matches the provided `supplier_id`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use uuid::Uuid;
    /// # use product_catalog::db::ProductRepo;
    /// # async fn example(repo: &ProductRepo) -> Result<(), sqlx::Error> {
    /// let supplier = Uuid::new_v4();
    /// let products = repo.get_by_supplier(supplier).await?;
    /// assert!(products.iter().all(|p| p.supplier_id == supplier));
    /// # Ok(())
    /// # }
    /// ```
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

    /// Update product fields for a supplier and return the updated product.
    ///
    /// Fields set to `None` in `req` leave the existing values unchanged. If
    /// `req.quantity_change` is `Some(n)`, the stored quantity is incremented by `n`;
    /// otherwise the quantity is set from `req.quantity` when provided.
    ///
    /// # Returns
    ///
    /// `Product` representing the updated database row.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uuid::Uuid;
    /// # use crate::db::ProductRepo;
    /// # use crate::models::UpdateProductRequest;
    /// # async fn doc_example(repo: &ProductRepo) -> Result<(), sqlx::Error> {
    /// let supplier_id = Uuid::new_v4();
    /// let product_id = Uuid::new_v4();
    /// let req = UpdateProductRequest {
    ///     name: Some("Updated name".into()),
    ///     description: None,
    ///     category: None,
    ///     price: None,
    ///     unit: None,
    ///     quantity: None,
    ///     available: None,
    ///     quantity_change: Some(3),
    ///     low_stock_threshold: None,
    /// };
    /// let updated = repo.update_product(supplier_id, product_id, &req).await?;
    /// assert_eq!(updated.product_id, product_id);
    /// # Ok(())
    /// # }
    /// ```
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

    /// Deletes the product matching the given supplier and product IDs.
    ///
    /// Removes the product row from the database for the provided `supplier_id` and `product_id`.
    ///
    /// # Returns
    ///
    /// `u64` — the number of rows deleted (0 if no matching product was found, 1 if deleted).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uuid::Uuid;
    ///
    /// # async fn example(repo: &ProductRepo) -> Result<(), sqlx::Error> {
    /// let supplier = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    /// let product = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
    /// let deleted = repo.delete_product(supplier, product).await?;
    /// assert!(deleted <= 1);
    /// # Ok(()) }
    /// ```
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

    /// Creates multiple products within a single database transaction.
    ///
    /// Applies defaults for `product_id` (new UUID), `quantity` (0), `available` (true),
    /// and `low_stock_threshold` (10) when those fields are not provided on individual items.
    /// The operation is atomic: either all products are inserted and the transaction is committed,
    /// or an error causes the transaction to be rolled back.
    ///
    /// Returns the list of inserted `Product` records in the same order as the input items.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // CreateProductRequest fields shown for illustration; adapt to your types.
    /// let items = vec![
    ///     CreateProductRequest {
    ///         supplier_id: some_supplier_id,
    ///         name: "Widget".into(),
    ///         description: "A useful widget".into(),
    ///         category: "tools".into(),
    ///         price: 9.99,
    ///         unit: "each".into(),
    ///         product_id: None,
    ///         quantity: None,
    ///         available: None,
    ///         low_stock_threshold: None,
    ///     },
    /// ];
    ///
    /// let created = repo.bulk_create(&items).await?;
    /// assert_eq!(created.len(), items.len());
    /// ```
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

    /// Registers a product asset entry and returns the created `ProductAsset`.
    ///
    /// If `req.is_primary` is true, existing assets for the given supplier and product will have their
    /// `is_primary` flag cleared before the new asset is inserted. The `provider` defaults to
    /// `"cloudinary"` when not provided.
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error::RowNotFound` if no product exists matching the `supplier_id` and
    /// `product_id`. Other database failures return the corresponding `sqlx::Error`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uuid::Uuid;
    /// # async fn doc(pool: &crate::PgPool) -> Result<(), sqlx::Error> {
    /// let repo = crate::ProductRepo::new(pool.clone());
    /// let supplier = Uuid::new_v4();
    /// let product = Uuid::new_v4();
    /// let req = crate::RegisterProductAssetRequest {
    ///     provider: None,
    ///     public_id: "public-id".into(),
    ///     url: "http://example.com/image.jpg".into(),
    ///     secure_url: "https://example.com/image.jpg".into(),
    ///     width: Some(800),
    ///     height: Some(600),
    ///     bytes: Some(150_000),
    ///     format: Some("jpg".into()),
    ///     alt_text: Some("Example image".into()),
    ///     is_primary: Some(true),
    /// };
    /// let asset = repo.register_product_asset(supplier, product, &req).await?;
    /// assert_eq!(asset.product_id, product);
    /// # Ok(()) }
    /// ```
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

    /// Returns the stored product asset metadata for a specific product.
    ///
    /// Results are ordered with `is_primary` assets first, then by `created_at` descending (newest first).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uuid::Uuid;
    /// # async fn example(repo: &crate::db::ProductRepo) -> Result<(), sqlx::Error> {
    /// let supplier_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    /// let product_id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
    /// let assets = repo.list_product_assets(supplier_id, product_id).await?;
    /// assert!(assets.iter().all(|a| a.product_id == product_id));
    /// # Ok(()) }
    /// ```
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

    /// Delete a product asset row matching the given supplier, product, and asset IDs.
    ///
    /// Returns the number of rows deleted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uuid::Uuid;
    ///
    /// # async fn example(repo: &crate::db::ProductRepo) {
    /// let supplier_id = Uuid::new_v4();
    /// let product_id = Uuid::new_v4();
    /// let asset_id = Uuid::new_v4();
    ///
    /// let deleted = repo.delete_product_asset(supplier_id, product_id, asset_id).await.unwrap();
    /// // `deleted` is the number of rows removed (0 if none matched)
    /// # }
    /// ```
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
