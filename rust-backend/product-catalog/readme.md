# Product Catalog Service

A Rust-based microservice for managing product data. It provides CRUD operations, bulk creation, search capabilities, and supplier-specific views. The service is designed to integrate with other microservices, publishing product events over Redis for real-time updates.

---

## Table of Contents

- [Product Catalog Service](#product-catalog-service)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Tech Stack](#tech-stack)
  - [Getting Started](#getting-started)
  - [Environment Variables](#environment-variables)
  - [Database Setup](#database-setup)
  - [Redis Setup](#redis-setup)
  - [API Endpoints](#api-endpoints)
  - [Example Requests](#example-requests)
  - [Event Publishing](#event-publishing)

---

## Features

* Create, update, delete, and retrieve products.
* Bulk product creation.
* Search products by category, price range, and supplier.
* Supplier-specific product listing.
* Publishes product events (`created`, `updated`, `deleted`) to Redis for integration with other services.

---

## Tech Stack

* **Rust** with [Actix Web](https://actix.rs/)
* **PostgreSQL** via [SQLx](https://docs.rs/sqlx/)
* **Redis** for event publishing
* **Serde** for JSON serialization/deserialization

---

## Getting Started

1. Clone the repository:

```bash
git clone https://github.com/your-org/product-catalog.git
cd product-catalog
```

2. Install dependencies and build:

```bash
cargo build --release
```

3. Set up the environment variables (see below).

4. Run migrations:

```bash
cargo run --bin product-catalog
```

---

## Environment Variables

| Variable       | Description                               | Default    |
| -------------- | ----------------------------------------- | ---------- |
| `DATABASE_URL` | PostgreSQL connection URL                 | *required* |
| `REDIS_URL`    | Redis connection URL for event publishing | optional   |
| `SERVICE_PORT` | Port for the HTTP server                  | `3003`     |

---

## Database Setup

* Ensure PostgreSQL is running and accessible.
* Run migrations:

```bash
sqlx migrate run
```

* The main table is `products` with the following schema:

```sql
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    price DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    unit TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    available BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## Redis Setup

* Redis is used for publishing events (`product.created`, `product.updated`, `product.deleted`).
* If `REDIS_URL` is not set or Redis is unavailable, the service continues in no-op mode.

---

## API Endpoints

| Method | Path                                   | Description                                                            |
| ------ | -------------------------------------- | ---------------------------------------------------------------------- |
| POST   | `/products`                            | Create a new product                                                   |
| POST   | `/products/bulk`                       | Create multiple products in bulk                                       |
| GET    | `/products/search`                     | Search products by `category`, `min_price`, `max_price`, `supplier_id` |
| GET    | `/products/{supplier_id}/{product_id}` | Retrieve a single product by supplier and product IDs                  |
| PUT    | `/products/{supplier_id}/{product_id}` | Update a product by supplier and product IDs                           |
| DELETE | `/products/{supplier_id}/{product_id}` | Delete a product by supplier and product IDs                           |
| GET    | `/products/{supplier_id}`              | List all products for a given supplier                                 |

---

## Example Requests

**Create Product**

```bash
POST /products
Content-Type: application/json

{
  "supplier_id": "b13a6cd4-7ff5-49cc-9c6c-0dc22a2b1d4b",
  "name": "Wireless Mouse",
  "description": "Ergonomic wireless mouse",
  "category": "Accessories",
  "price": 25.99,
  "unit": "pcs",
  "quantity": 50,
  "available": true
}
```

**Search Products**

```bash
GET /products/search?category=Accessories&min_price=10&max_price=50
```

**Update Product**

```bash
PUT /products/{supplier_id}/{product_id}
Content-Type: application/json

{
  "name": "Gaming Mouse",
  "quantity_change": -3
}
```

**Delete Product**

```bash
DELETE /products/{supplier_id}/{product_id}
```

---

## Event Publishing

* Product events are published over Redis for other services to consume.
* Event types:

  * `product.created`
  * `product.updated`
  * `product.deleted`
* Example event payload:

```json
{
  "event_type": "product.created",
  "product_id": "e5b3a2a1-8c49-4b1f-b90f-40a67d47f18c5",
  "supplier_id": "b13a6cd4-7ff5-49cc-9c6c-0dc22a2b1d4b",
  "name": "Wireless Mouse",
  "quantity": 50,
  "unit": "pcs"
}
```

---
