# 🏷️ Inventory Management Service

This service handles **inventory tracking** for products supplied by different vendors.  
It’s built with **Rust**, using **Actix-Web** for the API layer and **SQLx** for database operations.  
Redis is optionally used for event publishing and cache invalidation.

---

## 🧱 Tech Stack

| Layer | Tool | Purpose |
|-------|------|----------|
| Web Framework | **Actix-Web** | Handles routing and request/response logic |
| Database ORM | **SQLx** | Async, compile-time checked SQL queries |
| Database | **PostgreSQL** | Stores products and stock data |
| Caching/Event Bus | **Redis** | Publishes low-stock and update events |
| Serialization | **Serde** | Converts between JSON ↔ Rust structs |
| Time | **Chrono** | For handling `TIMESTAMPTZ` fields safely |

---

## ⚙️ Project Setup

1. **Clone the repository**
    ```bash
    git clone <repo-url>
    cd inventory-service
    ```

2. **Set up environment variables**

   Create a `.env` file in the root:

   ```bash
   DATABASE_URL=postgres://username:password@localhost/inventory_db
   REDIS_URL=redis://localhost:6379
   SERVICE_PORT=3002
   ```

3. **Run database migrations**

   ```bash
   sqlx migrate run
   ```

4. **Start the service**

   ```bash
   cargo run
   ```

   The API will run at:

   ```
   http://localhost:3002
   ```

---

## 📚 API Documentation

### 1. 🧾 Get all products for a supplier

**Route:**

```
GET /inventory/{supplier_id}
```

**Example:**

```
GET http://localhost:3002/inventory/11111111-1111-1111-1111-111111111111
```

**Response Example:**

```json
[
  {
    "product_id": "33333333-3333-3333-3333-333333333333",
    "supplier_id": "11111111-1111-1111-1111-111111111111",
    "name": "Rice bag",
    "quantity": 50,
    "low_stock_threshold": 10,
    "unit": "bags",
    "updated_at": "2025-11-01T14:00:00Z"
  },
  {
    "product_id": "22222222-2222-2222-2222-222222222222",
    "supplier_id": "11111111-1111-1111-1111-111111111111",
    "name": "Rice bag",
    "quantity": 50,
    "low_stock_threshold": 10,
    "unit": "bags",
    "updated_at": "2025-11-01T14:00:00Z"
  }
]
```

---

### 2. 📦 Create (Add) a New Product

**Route:**

```
POST /inventory
```

**Example Body:**

```json
{
  "product_id": "33333333-3333-3333-3333-333333333333",
  "supplier_id": "11111111-1111-1111-1111-111111111111",
  "quantity": 50,
  "name": "Rice bag",
  "low_stock_threshold": 10,
  "unit": "bags"
}
```

**Example cURL:**

```bash
curl -X POST http://localhost:3002/inventory \
-H "Content-Type: application/json" \
-d '{
  "product_id": "33333333-3333-3333-3333-333333333333",
  "supplier_id": "11111111-1111-1111-1111-111111111111",
  "quantity": 50,
  "name": "Rice bag",
  "low_stock_threshold": 10,
  "unit": "bags"
}'
```

**Response Example:**

```json
{
  "product_id": "33333333-3333-3333-3333-333333333333",
  "supplier_id": "11111111-1111-1111-1111-111111111111",
  "quantity": 50,
  "name": "Rice bag",
  "low_stock_threshold": 10,
  "unit": "bags",
  "updated_at": "2025-11-01T14:00:00Z"
}
```

---

### 3. 🔄 Update a Product’s Quantity

**Route:**

```
POST /inventory/{supplier_id}/update
```

**Example Body:**

```json
{
  "product_id": "22222222-2222-2222-2222-222222222222",
  "quantity_change": -12
}
```

**Example cURL:**

```bash
curl -X POST http://localhost:3002/inventory/11111111-1111-1111-1111-111111111111/update \
-H "Content-Type: application/json" \
-d '{
  "product_id": "22222222-2222-2222-2222-222222222222",
  "quantity_change": -12
}'
```

**Behavior:**

* Updates the product’s `quantity` by adding `quantity_change` (e.g., `-12` → subtracts 12).
* Publishes to Redis channel `inventory.updated`.
* If new stock ≤ `low_stock_threshold`, also publishes to `inventory.lowstock`.
* Invalidates Redis cache for that supplier.

**Response Example:**

```json
{
  "product_id": "22222222-2222-2222-2222-222222222222",
  "supplier_id": "11111111-1111-1111-1111-111111111111",
  "quantity": 38,
  "low_stock_threshold": 10,
  "unit": "bags",
  "updated_at": "2025-11-01T14:30:00Z"
}
```

---

### 4. 🔍 Get a Single Product by Supplier and Product ID

**Route:**

```
GET /inventory/{supplier_id}/{product_id}
```

**Example:**

```
GET http://localhost:3002/inventory/11111111-1111-1111-1111-111111111111/33333333-3333-3333-3333-333333333333
```

**Response Example:**

```json
{
  "product_id": "33333333-3333-3333-3333-333333333333",
  "supplier_id": "11111111-1111-1111-1111-111111111111",
  "name": "Rice bag",
  "quantity": 50,
  "low_stock_threshold": 10,
  "unit": "bags",
  "updated_at": "2025-11-01T14:00:00Z"
}
```

---

## 🧠 Notes & Gotchas (Don’t Forget!)

* **Always use `TIMESTAMPTZ` in PostgreSQL** and `DateTime<Utc>` in Rust to avoid type mismatch errors.
  `NaiveDateTime` only works for non-timezone timestamps.

* **Don’t put query data in the URL** for POST — use a JSON body.

* When testing with `curl`, always include:

  ```
  -H "Content-Type: application/json"
  ```

* A **404 Not Found** usually means the combination of `supplier_id` and `product_id` doesn’t exist in your DB — not that the route is broken.

* Redis publishing is optional, but if you’re using it, ensure the Redis server is running, or `.unwrap()` calls will panic.

* This service can easily scale horizontally since Redis acts as a central event bus.

---

## 🧩 Folder Structure (Simplified)

```
src/
├── main.rs            # Actix server setup
├── handlers.rs        # HTTP route handlers
├── models.rs          # Structs (InventoryItem, UpdateStockRequest)
├── db.rs              # Repository functions for SQLx
└── redis_pub.rs           # Redis publisher + cache handling
```

---

## 🧪 Example Workflow

1. Add a product via `/inventory`
2. Verify with `GET /inventory/{supplier_id}`
3. Update stock via `/inventory/{supplier_id}/update`
4. Check the change with `GET /inventory/{supplier_id}/{product_id}`

---

## 📬 Future Improvements

* Add authentication (JWT or API key)
* Add delete endpoint (`DELETE /inventory/{supplier_id}/{product_id}`)
* Integrate metrics (Prometheus or OpenTelemetry)
* Add pagination for large supplier inventories

---

**Author:** Inventory Microservice — Rust + Actix + SQLx
**Port:** `3002`
**Status:** Stable
