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
    - [Environment Variables](#environment-variables-1)
    - [Contributing](#contributing)

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
git clone https://github.com/relayyl6/product-catalog.git
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

Here’s a polished version you can put in your README under a “Configuration” or “Environment Variables” section:

---

### Environment Variables

Before running the service, create a `.env` file in the project root with the following values:

```env
# PostgreSQL connection URL
DATABASE_URL=your_postgres_db

# Redis connection URL
REDIS_URL=redis://127.0.0.1:6379

# Service port (default: 3003)
SERVICE_PORT=3003
```

Here’s a concise, developer-friendly “Contributing” section you can add to your README:

---

### Contributing

We welcome contributions to improve the Product Catalog service! Here’s how you can help:

1. **Fork the Repository**
   Click “Fork” at the top of the repository to create your own copy.

2. **Clone Your Fork**

   ```bash
   git clone https://github.com/relayyl6/product-catalog.git
   cd product-catalog
   ```

3. **Create a Feature Branch**

   ```bash
   git checkout -b feature/your-feature-name
   ```

4. **Set Up the Environment**

   * Create a `.env` file as described in the configuration section.
   * Ensure you have a running PostgreSQL database and Redis server.

5. **Make Your Changes**

   * Implement new features, fix bugs, or improve documentation.
   * Example areas for contribution:

     * Adding support for product images
     * Improving Redis event handling
     * Enhancing search functionality
     * Fixing typos or documentation

6. **Run Tests & Verify**
   Make sure all existing and new tests pass. For example:

   ```bash
   cargo test
   ```

7. **Commit Your Changes**

   ```bash
   git add .
   git commit -m "Add feature: your-feature-description"
   ```

8. **Push & Create a Pull Request**

   ```bash
   git push origin feature/your-feature-name
   ```

   * Go to your fork on GitHub and create a Pull Request to the main repository.
   * Provide a clear description of your changes and why they are beneficial.

9. **Iterate Based on Feedback**
   Maintainers may request changes. Update your branch and push again until the PR is ready to merge.

> 💡 **Tip:** Keep commits small and descriptive, and make sure your code follows Rust best practices.

---

