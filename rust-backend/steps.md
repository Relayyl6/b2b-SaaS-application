# Comprehensive B2B SaaS Backend Testing & Execution Guide (Master Blueprint)

This document is the **Master Testing Blueprint** for the entire B2B SaaS E-Commerce platform. It maps out **EVERY SINGLE ROUTE** across all 9 microservices, detailing the exact `cURL` commands with maximum possible payloads, the **EXPLICIT JSON responses**, and the **Event Mesh Subscriptions**.

---

## 1. Environment & Infrastructure Setup

### Required Infrastructure (Docker Compose)
We use Docker Compose to run the foundational data layer.
```bash
docker-compose up -d postgres redis rabbitmq
```

### The `.env` Configuration
Copy `.env.example` to `.env` in the project root. Fill in your `DATABASE_URL` (e.g., Neon Postgres).

---

## 2. Event Mesh Architecture (Pub/Sub)

* **Payments Service** listens to `order.created`, `order.cancelled`, `logistics.shipment_updated`.
* **Inventory Service** listens to `order.created`, `payment.failed`, `order.cancelled`, `order.confirmed`.
* **Logistics Service** listens to `order.confirmed`.
* **Order Service** listens to `payment.succeeded`, `payment.failed`, `logistics.shipment_updated`.
* **Notifications Service** listens to `user.created`, `order.confirmed`, `logistics.shipment_updated`.
* **Analytics Service** listens to **ALL EVENTS**.

---

## 3. MASTER ROUTE MAP & TESTING GUIDE

*(Note: Replace placeholders like `YOUR_ACCESS_TOKEN`, `YOUR_USER_ID`, etc., with actual UUIDs).*

### A. User Management Service (Port 3001)

#### 1. Sign Up (POST `/signup`)
```bash
curl -X POST http://localhost:3001/signup -H "Content-Type: application/json" -d '{"email": "admin@b2bsaas.com", "password": "SecurePassword123!", "full_name": "John Doe", "role": "Admin"}'
```
**Response** (`200 OK`):
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "email": "admin@b2bsaas.com",
  "full_name": "John Doe",
  "role": "Admin",
  "is_active": true,
  "email_verified": false,
  "created_at": "2026-07-24T22:51:00Z",
  "updated_at": "2026-07-24T22:51:00Z"
}
```

#### 2. Sign In (POST `/signin`)
```bash
curl -X POST http://localhost:3001/signin -H "Content-Type: application/json" -d '{"email": "admin@b2bsaas.com", "password": "SecurePassword123!"}'
```
**Response** (`200 OK`):
```json
{
  "user": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "admin@b2bsaas.com",
    "full_name": "John Doe",
    "role": "Admin",
    "is_active": true,
    "email_verified": false,
    "created_at": "2026-07-24T22:51:00Z",
    "updated_at": "2026-07-24T22:51:00Z"
  },
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjNlNDU2Ny1lODliLTEyZDMtYTQ1Ni00MjY2MTQxNzQwMDAiLCJleHAiOjE2OTE0MDUyMTR9.D_M9o..."
}
```

#### 3. Sign Out (POST `/signout`)
```bash
curl -X POST http://localhost:3001/signout -H "Authorization: Bearer YOUR_ACCESS_TOKEN" -H "Content-Type: application/json" -d '{"token": "YOUR_ACCESS_TOKEN"}'
```
**Response** (`200 OK`):
```text
Successfully signed out
```

#### 4. Validate Token (GET `/auth/validate`)
```bash
curl -X GET http://localhost:3001/auth/validate -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```
**Response** (`200 OK`):
```text
Token is valid
```

#### 5. Get User Profile (GET `/get_user/{id}`)
```bash
curl -X GET http://localhost:3001/get_user/123e4567-e89b-12d3-a456-426614174000 -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```
**Response** (`200 OK`):
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "email": "admin@b2bsaas.com",
  "full_name": "John Doe",
  "role": "Admin",
  "is_active": true,
  "email_verified": true,
  "created_at": "2026-07-24T22:51:00Z",
  "updated_at": "2026-07-25T14:30:00Z"
}
```

#### 6. Update User (PUT `/protected/update/{id}`)
```bash
curl -X PUT http://localhost:3001/protected/update/123e4567-e89b-12d3-a456-426614174000 -H "Authorization: Bearer YOUR_ACCESS_TOKEN" -H "Content-Type: application/json" -d '{"full_name": "Jane Doe Updated", "is_active": true}'
```
**Response** (`200 OK`):
```text
User updated successfully
```

#### 7. Delete User (DELETE `/protected/delete/{id}`)
```bash
curl -X DELETE http://localhost:3001/protected/delete/123e4567-e89b-12d3-a456-426614174000 -H "Authorization: Bearer YOUR_ACCESS_TOKEN" -H "Content-Type: application/json" -d '{"user_id": "123e4567-e89b-12d3-a456-426614174000"}'
```
**Response** (`200 OK`):
```text
User deleted successfully
```

#### 8. Admin Stats (GET `/admin/stats`)
```bash
curl -X GET http://localhost:3001/admin/stats -H "Authorization: Bearer YOUR_ADMIN_ACCESS_TOKEN"
```
**Response** (`200 OK`):
```json
{
  "total_users": 1500,
  "active_users": 1400,
  "new_users_today": 25
}
```

#### 9. Forgot Password (POST `/forgot-password`)
```bash
curl -X POST http://localhost:3001/forgot-password -H "Content-Type: application/json" -d '{"email": "admin@b2bsaas.com"}'
```
**Response** (`200 OK`):
```text
If that email is registered, a password reset link has been sent.
```

#### 10. Reset Password (POST `/reset-password`)
```bash
curl -X POST http://localhost:3001/reset-password -H "Content-Type: application/json" -d '{"token": "RESET_TOKEN", "new_password": "NewSecurePassword123!"}'
```
**Response** (`200 OK`):
```text
Password reset successfully
```

#### 11. Verify Email (POST `/verify-email`)
```bash
curl -X POST http://localhost:3001/verify-email -H "Content-Type: application/json" -d '{"token": "VERIFY_TOKEN"}'
```
**Response** (`200 OK`):
```text
Email verified successfully
```

---

### B. Supplier Management Service (Port 3004)

## 1. Health Check
- **Route:** `GET /health`
- **Description:** Returns the health status of the service.
- **Event Mesh Triggers:** None.

**cURL Command:**
```bash
curl -X GET "http://localhost:3004/health" \
  -H "Accept: application/json"
```

**Expected Response:**
```json
{
  "status": "ok",
  "service": "supplier-management"
}
```

## 2. Metrics
- **Route:** `GET /metrics`
- **Description:** Exposes Prometheus metrics for the service.
- **Event Mesh Triggers:** None.

**cURL Command:**
```bash
curl -X GET "http://localhost:3004/metrics"
```

**Expected Response:**
```text
# HELP ...
# TYPE ...
(Returns standard Prometheus metrics payload)
```

## 3. Create Supplier
- **Route:** `POST /suppliers`
- **Description:** Creates a new supplier.
- **Event Mesh Triggers:** 
  - **Publishes:** `supplier.created` event to the stream.
  - **Subscribes:** None.

**cURL Command:**
```bash
curl -X POST "http://localhost:3004/suppliers" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{
    "owner_user_id": "123e4567-e89b-12d3-a456-426614174000",
    "legal_name": "Acme Corp Ltd",
    "display_name": "Acme Supplies",
    "tax_id": "GB123456789",
    "country": "GB",
    "metadata": {
      "website": "https://acme-supplies.com",
      "contact_email": "sales@acme-supplies.com"
    },
    "platform_fee_percent": 5.5
  }'
```

**Expected Response:**
```json
{
  "id": "223e4567-e89b-12d3-a456-426614174001",
  "owner_user_id": "123e4567-e89b-12d3-a456-426614174000",
  "legal_name": "Acme Corp Ltd",
  "display_name": "Acme Supplies",
  "tax_id": "GB123456789",
  "country": "GB",
  "status": "pending",
  "stripe_account_id": null,
  "platform_fee_percent": 5.5,
  "metadata": {
    "website": "https://acme-supplies.com",
    "contact_email": "sales@acme-supplies.com"
  },
  "created_at": "2023-10-12T07:20:50.520Z",
  "updated_at": "2023-10-12T07:20:50.520Z"
}
```

## 4. Get Supplier
- **Route:** `GET /suppliers/{id}`
- **Description:** Retrieves a single supplier by their ID.
- **Event Mesh Triggers:** None.

**cURL Command:**
```bash
curl -X GET "http://localhost:3004/suppliers/223e4567-e89b-12d3-a456-426614174001" \
  -H "Accept: application/json"
```

**Expected Response:**
```json
{
  "id": "223e4567-e89b-12d3-a456-426614174001",
  "owner_user_id": "123e4567-e89b-12d3-a456-426614174000",
  "legal_name": "Acme Corp Ltd",
  "display_name": "Acme Supplies",
  "tax_id": "GB123456789",
  "country": "GB",
  "status": "pending",
  "stripe_account_id": "acct_1032D82eB675G8Z",
  "platform_fee_percent": 5.5,
  "metadata": {
    "website": "https://acme-supplies.com",
    "contact_email": "sales@acme-supplies.com"
  },
  "created_at": "2023-10-12T07:20:50.520Z",
  "updated_at": "2023-10-12T07:20:50.520Z"
}
```

## 5. List Owner Suppliers
- **Route:** `GET /suppliers/owner/{owner_user_id}`
- **Description:** Lists all suppliers owned by a specific user.
- **Event Mesh Triggers:** None.

**cURL Command:**
```bash
curl -X GET "http://localhost:3004/suppliers/owner/123e4567-e89b-12d3-a456-426614174000" \
  -H "Accept: application/json"
```

**Expected Response:**
```json
[
  {
    "id": "223e4567-e89b-12d3-a456-426614174001",
    "owner_user_id": "123e4567-e89b-12d3-a456-426614174000",
    "legal_name": "Acme Corp Ltd",
    "display_name": "Acme Supplies",
    "tax_id": "GB123456789",
    "country": "GB",
    "status": "active",
    "stripe_account_id": "acct_1032D82eB675G8Z",
    "platform_fee_percent": 5.5,
    "metadata": {
      "website": "https://acme-supplies.com",
      "contact_email": "sales@acme-supplies.com"
    },
    "created_at": "2023-10-12T07:20:50.520Z",
    "updated_at": "2023-10-12T08:30:15.100Z"
  }
]
```

## 6. Update Supplier
- **Route:** `PUT /suppliers/{id}`
- **Description:** Updates supplier details.
- **Event Mesh Triggers:**
  - **Publishes:** `supplier.updated` event to the stream.
  - **Subscribes:** None.

**cURL Command:**
```bash
curl -X PUT "http://localhost:3004/suppliers/223e4567-e89b-12d3-a456-426614174001" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -H "X-User-Id: 123e4567-e89b-12d3-a456-426614174000" \
  -d '{
    "legal_name": "Acme Corporation Limited",
    "display_name": "Acme Supplies & Co",
    "tax_id": "GB987654321",
    "country": "UK",
    "platform_fee_percent": 4.5,
    "metadata": {
      "website": "https://acme-supplies.co.uk",
      "contact_email": "hello@acme-supplies.co.uk"
    }
  }'
```

**Expected Response:**
```json
{
  "id": "223e4567-e89b-12d3-a456-426614174001",
  "owner_user_id": "123e4567-e89b-12d3-a456-426614174000",
  "legal_name": "Acme Corporation Limited",
  "display_name": "Acme Supplies & Co",
  "tax_id": "GB987654321",
  "country": "UK",
  "status": "active",
  "stripe_account_id": "acct_1032D82eB675G8Z",
  "platform_fee_percent": 4.5,
  "metadata": {
    "website": "https://acme-supplies.co.uk",
    "contact_email": "hello@acme-supplies.co.uk"
  },
  "created_at": "2023-10-12T07:20:50.520Z",
  "updated_at": "2023-10-12T09:45:22.000Z"
}
```

## 7. Update Supplier Status
- **Route:** `PUT /suppliers/{id}/status`
- **Description:** Updates the status of a supplier.
- **Event Mesh Triggers:**
  - **Publishes:** `supplier.status_updated` event to the stream.
  - **Subscribes:** None.

**cURL Command:**
```bash
curl -X PUT "http://localhost:3004/suppliers/223e4567-e89b-12d3-a456-426614174001/status" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -H "X-User-Id: 123e4567-e89b-12d3-a456-426614174000" \
  -d '{
    "status": "active"
  }'
```

**Expected Response:**
```json
{
  "id": "223e4567-e89b-12d3-a456-426614174001",
  "owner_user_id": "123e4567-e89b-12d3-a456-426614174000",
  "legal_name": "Acme Corporation Limited",
  "display_name": "Acme Supplies & Co",
  "tax_id": "GB987654321",
  "country": "UK",
  "status": "active",
  "stripe_account_id": "acct_1032D82eB675G8Z",
  "platform_fee_percent": 4.5,
  "metadata": {
    "website": "https://acme-supplies.co.uk",
    "contact_email": "hello@acme-supplies.co.uk"
  },
  "created_at": "2023-10-12T07:20:50.520Z",
  "updated_at": "2023-10-12T09:50:00.000Z"
}
```

---

### C. Product Catalog Service (Port 3003)

## 1. GET /metrics
**Description:** Retrieves service metrics.
**cURL:**
```bash
curl -X GET http://localhost:3003/metrics \
  -H "Accept: application/json"
```
**Expected Response:** (Prometheus text format, not defined in models)
**Event Mesh Triggers:** None

## 2. POST /products
**Description:** Creates a product and emits best-effort integration events.
**cURL:**
```bash
curl -X POST http://localhost:3003/products \
  -H "Content-Type: application/json" \
  -d '{
    "product_id": "123e4567-e89b-12d3-a456-426614174000",
    "supplier_id": "123e4567-e89b-12d3-a456-426614174001",
    "name": "Super Widget",
    "description": {"detail": "A very super widget"},
    "category": "Widgets",
    "price": 99.99,
    "unit": "piece",
    "quantity": 100,
    "available": true,
    "low_stock_threshold": 10,
    "sku": "WIDGET-001",
    "variants": {"color": "red"}
  }'
```
**Expected Response:**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174002",
  "product_id": "123e4567-e89b-12d3-a456-426614174000",
  "supplier_id": "123e4567-e89b-12d3-a456-426614174001",
  "name": "Super Widget",
  "description": {"detail": "A very super widget"},
  "category": "Widgets",
  "price": 99.99,
  "unit": "piece",
  "quantity": 100,
  "available": true,
  "low_stock_threshold": 10,
  "sku": "WIDGET-001",
  "variants": {"color": "red"},
  "created_at": "2026-07-24T22:51:07Z",
  "updated_at": "2026-07-24T22:51:07Z",
  "deleted_at": null
}
```
**Event Mesh Triggers:** Publishes `product.created` event.

## 3. POST /products/bulk
**Description:** Creates products in bulk and emits events.
**cURL:**
```bash
curl -X POST http://localhost:3003/products/bulk \
  -H "Content-Type: application/json" \
  -d '{
    "products": [
      {
        "product_id": "123e4567-e89b-12d3-a456-426614174000",
        "supplier_id": "123e4567-e89b-12d3-a456-426614174001",
        "name": "Super Widget",
        "description": {"detail": "A very super widget"},
        "category": "Widgets",
        "price": 99.99,
        "unit": "piece",
        "quantity": 100,
        "available": true,
        "low_stock_threshold": 10,
        "sku": "WIDGET-001",
        "variants": {"color": "red"}
      }
    ]
  }'
```
**Expected Response:**
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174002",
    "product_id": "123e4567-e89b-12d3-a456-426614174000",
    "supplier_id": "123e4567-e89b-12d3-a456-426614174001",
    "name": "Super Widget",
    "description": {"detail": "A very super widget"},
    "category": "Widgets",
    "price": 99.99,
    "unit": "piece",
    "quantity": 100,
    "available": true,
    "low_stock_threshold": 10,
    "sku": "WIDGET-001",
    "variants": {"color": "red"},
    "created_at": "2026-07-24T22:51:07Z",
    "updated_at": "2026-07-24T22:51:07Z",
    "deleted_at": null
  }
]
```
**Event Mesh Triggers:** Publishes `product.created` event for each product.

## 4. GET /products/search
**Description:** Searches products by optional query parameters.
**cURL:**
```bash
curl -X GET "http://localhost:3003/products/search?category=Widgets&min_price=10.0&max_price=100.0&supplier_id=123e4567-e89b-12d3-a456-426614174001&product_id=123e4567-e89b-12d3-a456-426614174000&limit=50&offset=0" \
  -H "Accept: application/json"
```
**Expected Response:**
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174002",
    "product_id": "123e4567-e89b-12d3-a456-426614174000",
    "supplier_id": "123e4567-e89b-12d3-a456-426614174001",
    "name": "Super Widget",
    "description": {"detail": "A very super widget"},
    "category": "Widgets",
    "price": 99.99,
    "unit": "piece",
    "quantity": 100,
    "available": true,
    "low_stock_threshold": 10,
    "sku": "WIDGET-001",
    "variants": {"color": "red"},
    "created_at": "2026-07-24T22:51:07Z",
    "updated_at": "2026-07-24T22:51:07Z",
    "deleted_at": null
  }
]
```
**Event Mesh Triggers:** None

## 5. GET /products/{supplier_id}/{product_id}
**Description:** Returns a single product by supplier and product id.
**cURL:**
```bash
curl -X GET http://localhost:3003/products/123e4567-e89b-12d3-a456-426614174001/123e4567-e89b-12d3-a456-426614174000 \
  -H "Accept: application/json"
```
**Expected Response:**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174002",
  "product_id": "123e4567-e89b-12d3-a456-426614174000",
  "supplier_id": "123e4567-e89b-12d3-a456-426614174001",
  "name": "Super Widget",
  "description": {"detail": "A very super widget"},
  "category": "Widgets",
  "price": 99.99,
  "unit": "piece",
  "quantity": 100,
  "available": true,
  "low_stock_threshold": 10,
  "sku": "WIDGET-001",
  "variants": {"color": "red"},
  "created_at": "2026-07-24T22:51:07Z",
  "updated_at": "2026-07-24T22:51:07Z",
  "deleted_at": null
}
```
**Event Mesh Triggers:** None

## 6. PUT /products/{supplier_id}/{product_id}
**Description:** Updates a product and emits a product.updated event.
**cURL:**
```bash
curl -X PUT http://localhost:3003/products/123e4567-e89b-12d3-a456-426614174001/123e4567-e89b-12d3-a456-426614174000 \
  -H "Content-Type: application/json" \
  -d '{
    "product_id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "Updated Super Widget",
    "description": {"detail": "An updated widget"},
    "category": "Widgets",
    "price": 89.99,
    "unit": "piece",
    "quantity": null,
    "available": false,
    "quantity_change": -10,
    "low_stock_threshold": 5,
    "sku": "WIDGET-002",
    "variants": {"color": "blue"}
  }'
```
**Expected Response:**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174002",
  "product_id": "123e4567-e89b-12d3-a456-426614174000",
  "supplier_id": "123e4567-e89b-12d3-a456-426614174001",
  "name": "Updated Super Widget",
  "description": {"detail": "An updated widget"},
  "category": "Widgets",
  "price": 89.99,
  "unit": "piece",
  "quantity": 90,
  "available": false,
  "low_stock_threshold": 5,
  "sku": "WIDGET-002",
  "variants": {"color": "blue"},
  "created_at": "2026-07-24T22:51:07Z",
  "updated_at": "2026-07-24T22:51:07Z",
  "deleted_at": null
}
```
**Event Mesh Triggers:** Publishes `product.updated` event. (Invalidates Redis cache for supplier products).

## 7. DELETE /products/{supplier_id}/{product_id}
**Description:** Deletes a product, emits product.deleted, and invalidates cache.
**cURL:**
```bash
curl -X DELETE http://localhost:3003/products/123e4567-e89b-12d3-a456-426614174001/123e4567-e89b-12d3-a456-426614174000
```
**Expected Response:** String response
```text
Product deleted successfully
```
**Event Mesh Triggers:** Publishes `product.deleted` event. (Invalidates Redis cache).

## 8. POST /products/{supplier_id}/{product_id}/assets
**Description:** Stores uploaded asset metadata for a product.
**cURL:**
```bash
curl -X POST http://localhost:3003/products/123e4567-e89b-12d3-a456-426614174001/123e4567-e89b-12d3-a456-426614174000/assets \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "cloudinary",
    "public_id": "sample_public_id",
    "url": "http://example.com/asset.jpg",
    "secure_url": "https://example.com/asset.jpg",
    "width": 800,
    "height": 600,
    "bytes": 102400,
    "format": "jpg",
    "alt_text": "Sample Asset",
    "is_primary": true
  }'
```
**Expected Response:**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174003",
  "product_id": "123e4567-e89b-12d3-a456-426614174000",
  "supplier_id": "123e4567-e89b-12d3-a456-426614174001",
  "provider": "cloudinary",
  "public_id": "sample_public_id",
  "url": "http://example.com/asset.jpg",
  "secure_url": "https://example.com/asset.jpg",
  "width": 800,
  "height": 600,
  "bytes": 102400,
  "format": "jpg",
  "alt_text": "Sample Asset",
  "is_primary": true,
  "created_at": "2026-07-24T22:51:07Z"
}
```
**Event Mesh Triggers:** None

## 9. GET /products/{supplier_id}/{product_id}/assets
**Description:** Lists stored asset metadata for a product.
**cURL:**
```bash
curl -X GET http://localhost:3003/products/123e4567-e89b-12d3-a456-426614174001/123e4567-e89b-12d3-a456-426614174000/assets \
  -H "Accept: application/json"
```
**Expected Response:**
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174003",
    "product_id": "123e4567-e89b-12d3-a456-426614174000",
    "supplier_id": "123e4567-e89b-12d3-a456-426614174001",
    "provider": "cloudinary",
    "public_id": "sample_public_id",
    "url": "http://example.com/asset.jpg",
    "secure_url": "https://example.com/asset.jpg",
    "width": 800,
    "height": 600,
    "bytes": 102400,
    "format": "jpg",
    "alt_text": "Sample Asset",
    "is_primary": true,
    "created_at": "2026-07-24T22:51:07Z"
  }
]
```
**Event Mesh Triggers:** None

## 10. DELETE /products/{supplier_id}/{product_id}/assets/{asset_id}
**Description:** Deletes product asset metadata by asset id.
**cURL:**
```bash
curl -X DELETE http://localhost:3003/products/123e4567-e89b-12d3-a456-426614174001/123e4567-e89b-12d3-a456-426614174000/assets/123e4567-e89b-12d3-a456-426614174003
```
**Expected Response:** String response
```text
Asset deleted
```
**Event Mesh Triggers:** None

## 11. POST /assets/cloudinary/sign-upload
**Description:** Generates signed Cloudinary upload parameters for direct client uploads.
**cURL:**
```bash
curl -X POST http://localhost:3003/assets/cloudinary/sign-upload \
  -H "Content-Type: application/json" \
  -d '{
    "folder": "b2b-saas/products",
    "public_id": "my_custom_asset_name"
  }'
```
**Expected Response:**
```json
{
  "cloud_name": "your_cloud_name",
  "api_key": "your_api_key",
  "timestamp": 1690240000,
  "signature": "a1b2c3d4e5f6g7h8i9j0",
  "folder": "b2b-saas/products",
  "public_id": "my_custom_asset_name"
}
```
**Event Mesh Triggers:** None

## 12. GET /products/{supplier_id}
**Description:** Returns all products for a supplier and emits view events.
**cURL:**
```bash
curl -X GET http://localhost:3003/products/123e4567-e89b-12d3-a456-426614174001 \
  -H "Accept: application/json"
```
**Expected Response:**
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174002",
    "product_id": "123e4567-e89b-12d3-a456-426614174000",
    "supplier_id": "123e4567-e89b-12d3-a456-426614174001",
    "name": "Super Widget",
    "description": {"detail": "A very super widget"},
    "category": "Widgets",
    "price": 99.99,
    "unit": "piece",
    "quantity": 100,
    "available": true,
    "low_stock_threshold": 10,
    "sku": "WIDGET-001",
    "variants": {"color": "red"},
    "created_at": "2026-07-24T22:51:07Z",
    "updated_at": "2026-07-24T22:51:07Z",
    "deleted_at": null
  }
]
```
**Event Mesh Triggers:** Publishes `product.viewed` event for each returned product.

---

### D. Inventory Management Service (Port 3006)

## Service-Level Event Subscriptions
The service listens to the following events from Redis Streams to manage its data asynchronously:
- `product.created`
- `product.updated`
- `product.deleted`
- `order.created`
- `order.cancelled`
- `order.failed`
- `payment.success`
- `payment.failed`
- `payment.cancelled`

---

## Routes

### 1. `GET /metrics`
Returns Prometheus metrics for the service.

**cURL Command:**
```bash
curl -X GET http://localhost:3006/metrics
```

**Expected Response:**
```text
# HELP inventory_management_http_requests_total Total HTTP requests
# TYPE inventory_management_http_requests_total counter
...
```

**Event Mesh Triggers:**
- Publishes: None
- Subscribes: None

---

### 2. `POST /inventory`
Creates a new inventory item.

**cURL Command:**
```bash
curl -X POST http://localhost:3006/inventory \
  -H "Content-Type: application/json" \
  -d '{
    "supplier_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
    "product_id": "c138d21c-4b68-4a94-a957-bf84f5bc9004",
    "name": "Widget Pro",
    "category": "Electronics",
    "description": {
      "key": "value",
      "details": "string"
    },
    "price": 99.99,
    "quantity": 100,
    "low_stock_threshold": 10,
    "unit": "pieces"
  }'
```

**Expected Response:**
```json
{
  "id": "e3b0c442-989b-4643-7313-132332132111",
  "supplier_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
  "product_id": "c138d21c-4b68-4a94-a957-bf84f5bc9004",
  "name": "Widget Pro",
  "description": {
    "key": "value",
    "details": "string"
  },
  "category": "Electronics",
  "price": 99.99,
  "quantity": 100,
  "low_stock_threshold": 10,
  "unit": "pieces",
  "available": true,
  "updated_at": "2026-07-24T22:51:06Z"
}
```

**Event Mesh Triggers:**
- Publishes: None explicitly from this route.
- Subscribes: None route-specific.

---

### 3. `GET /inventory/{supplier_id}/{product_id}`
Retrieves a specific inventory item by supplier ID and product ID.

**cURL Command:**
```bash
curl -X GET http://localhost:3006/inventory/d290f1ee-6c54-4b01-90e6-d701748f0851/c138d21c-4b68-4a94-a957-bf84f5bc9004
```

**Expected Response:**
```json
{
  "id": "e3b0c442-989b-4643-7313-132332132111",
  "supplier_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
  "product_id": "c138d21c-4b68-4a94-a957-bf84f5bc9004",
  "name": "Widget Pro",
  "description": {
    "key": "value",
    "details": "string"
  },
  "category": "Electronics",
  "price": 99.99,
  "quantity": 100,
  "low_stock_threshold": 10,
  "unit": "pieces",
  "available": true,
  "updated_at": "2026-07-24T22:51:06Z"
}
```

**Event Mesh Triggers:**
- Publishes: None
- Subscribes: None

---

### 4. `GET /inventory/{supplier_id}`
Retrieves all inventory items for a specific supplier.

**cURL Command:**
```bash
curl -X GET http://localhost:3006/inventory/d290f1ee-6c54-4b01-90e6-d701748f0851
```

**Expected Response:**
```json
[
  {
    "id": "e3b0c442-989b-4643-7313-132332132111",
    "supplier_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
    "product_id": "c138d21c-4b68-4a94-a957-bf84f5bc9004",
    "name": "Widget Pro",
    "description": {
      "key": "value",
      "details": "string"
    },
    "category": "Electronics",
    "price": 99.99,
    "quantity": 100,
    "low_stock_threshold": 10,
    "unit": "pieces",
    "available": true,
    "updated_at": "2026-07-24T22:51:06Z"
  }
]
```

**Event Mesh Triggers:**
- Publishes: None
- Subscribes: None

---

### 5. `POST /inventory/{supplier_id}/update`
Updates stock and details for a product.

**cURL Command:**
```bash
curl -X POST http://localhost:3006/inventory/d290f1ee-6c54-4b01-90e6-d701748f0851/update \
  -H "Content-Type: application/json" \
  -d '{
    "product_id": "c138d21c-4b68-4a94-a957-bf84f5bc9004",
    "name": "Widget Pro Updated",
    "description": {
      "key": "new_value"
    },
    "category": "Electronics",
    "price": 89.99,
    "unit": "pieces",
    "quantity": 90,
    "quantity_change": -10,
    "available": true,
    "low_stock_threshold": 10,
    "reserved": 0
  }'
```

**Expected Response:**
```json
{
  "id": "e3b0c442-989b-4643-7313-132332132111",
  "supplier_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
  "product_id": "c138d21c-4b68-4a94-a957-bf84f5bc9004",
  "name": "Widget Pro Updated",
  "description": {
    "key": "new_value"
  },
  "category": "Electronics",
  "price": 89.99,
  "quantity": 90,
  "low_stock_threshold": 10,
  "unit": "pieces",
  "available": true,
  "updated_at": "2026-07-24T22:51:06Z"
}
```

**Event Mesh Triggers:**
- Publishes to: 
  - `inventory.updated`
  - `inventory.lowstock` (conditionally, if `new_quantity <= low_stock_threshold`).
- Side Effect: Deletes cache key `inventory:supplier:{supplier_id}`.

---

### 6. `DELETE /inventory/{supplier_id}/{product_id}`
Deletes a product's inventory record.

**cURL Command:**
```bash
curl -X DELETE http://localhost:3006/inventory/d290f1ee-6c54-4b01-90e6-d701748f0851/c138d21c-4b68-4a94-a957-bf84f5bc9004
```

**Expected Response:**
```text
Product deleted successfully
```

**Event Mesh Triggers:**
- Publishes to: `inventory.deleted`
- Side Effect: Deletes cache key `inventory:supplier:{supplier_id}`.

---

### E. Order Service (Port 3005)

## Routes

### 1. Create Order
- **Endpoint:** `POST /orders`
- **Description:** Creates a new order.

#### 1.1 `curl` Command
```bash
curl -X POST http://localhost:3005/orders \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your_token>" \
  -d '{
    "user_id": "123e4567-e89b-12d3-a456-426614174000",
    "supplier_id": "223e4567-e89b-12d3-a456-426614174001",
    "product_id": "323e4567-e89b-12d3-a456-426614174002",
    "qty": 5,
    "status": "pending",
    "items": {
      "item_name": "Premium Widget",
      "variant": "Red",
      "weight": 1.2
    }
  }'
```

#### 1.2 JSON Expected Response
```json
{
  "message": "Order successfully created",
  "id": {
    "id": "423e4567-e89b-12d3-a456-426614174003",
    "product_id": "323e4567-e89b-12d3-a456-426614174002",
    "user_id": "123e4567-e89b-12d3-a456-426614174000",
    "supplier_id": "223e4567-e89b-12d3-a456-426614174001",
    "items": {
      "item_name": "Premium Widget",
      "variant": "Red",
      "weight": 1.2
    },
    "qty": 5,
    "status": "pending",
    "updated_at": null,
    "expires_at": "2026-07-26T22:51:17Z",
    "order_timestamp": "2026-07-24T22:51:17Z",
    "version": 1
  }
}
```

#### 1.3 Event Mesh Triggers
- **Publishes:** `order.created`

---

### 2. Get Order
- **Endpoint:** `GET /orders/{id}`
- **Description:** Retrieves an order by its ID.

#### 2.1 `curl` Command
```bash
curl -X GET http://localhost:3005/orders/423e4567-e89b-12d3-a456-426614174003 \
  -H "Authorization: Bearer <your_token>"
```

#### 2.2 JSON Expected Response
```json
{
  "id": "423e4567-e89b-12d3-a456-426614174003",
  "product_id": "323e4567-e89b-12d3-a456-426614174002",
  "user_id": "123e4567-e89b-12d3-a456-426614174000",
  "supplier_id": "223e4567-e89b-12d3-a456-426614174001",
  "items": {
    "item_name": "Premium Widget",
    "variant": "Red",
    "weight": 1.2
  },
  "qty": 5,
  "status": "pending",
  "updated_at": "2026-07-24T23:00:00Z",
  "expires_at": "2026-07-26T22:51:17Z",
  "order_timestamp": "2026-07-24T22:51:17Z",
  "version": 1
}
```

#### 2.3 Event Mesh Triggers
- **Publishes:** None

---

### 3. Update Order Status
- **Endpoint:** `PUT /orders/{id}/status`
- **Description:** Updates the status of an existing order using optimistic concurrency control.

#### 3.1 `curl` Command
```bash
curl -X PUT http://localhost:3005/orders/423e4567-e89b-12d3-a456-426614174003/status \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your_token>" \
  -d '{
    "id": "423e4567-e89b-12d3-a456-426614174003",
    "product_id": "323e4567-e89b-12d3-a456-426614174002",
    "user_id": "123e4567-e89b-12d3-a456-426614174000",
    "new_status": "cancelled",
    "expires_at": "2026-07-26T22:51:17Z",
    "order_timestamp": "2026-07-24T22:51:17Z",
    "expected_version": 1
  }'
```

#### 3.2 JSON Expected Response
```json
{
  "message": "Order status updated",
  "status": {
    "id": "423e4567-e89b-12d3-a456-426614174003",
    "product_id": "323e4567-e89b-12d3-a456-426614174002",
    "user_id": "123e4567-e89b-12d3-a456-426614174000",
    "supplier_id": "223e4567-e89b-12d3-a456-426614174001",
    "items": {
      "item_name": "Premium Widget",
      "variant": "Red",
      "weight": 1.2
    },
    "qty": 5,
    "status": "cancelled",
    "updated_at": "2026-07-24T23:05:00Z",
    "expires_at": "2026-07-26T22:51:17Z",
    "order_timestamp": "2026-07-24T22:51:17Z",
    "version": 2
  }
}
```

#### 3.3 Event Mesh Triggers
- **Publishes:** Depending on the `new_status`, it may publish:
  - `order.cancelled`
  - `order.shipped`
  - `order.refunded`

---

### 4. Delete Order
- **Endpoint:** `DELETE /orders/{id}/{user_id}`
- **Description:** Deletes an order from the database.

#### 4.1 `curl` Command
```bash
curl -X DELETE http://localhost:3005/orders/423e4567-e89b-12d3-a456-426614174003/123e4567-e89b-12d3-a456-426614174000 \
  -H "Authorization: Bearer <your_token>"
```

#### 4.2 Text Expected Response
```text
Order deleted successfully
```

#### 4.3 Event Mesh Triggers
- **Publishes:** None

---

### 5. Metrics
- **Endpoint:** `GET /metrics`
- **Description:** Exposes service metrics for Prometheus.

#### 5.1 `curl` Command
```bash
curl -X GET http://localhost:3005/metrics
```

#### 5.2 Expected Response
```text
# HELP ...
# TYPE ...
```

#### 5.3 Event Mesh Triggers
- **Publishes:** None

---

## Service-Level Event Mesh Subscriptions (Background Listener)
The `order-service` actively listens to the following events on Redis Streams:
- **Inventory Events:**
  - `inventory.rejected` (Updates status to Failed)
  - `inventory.reservation_expired`, `inventory.expired`, `inventory.released` (Updates status to Cancelled)
  - `inventory.reserved` (Updates status to Confirmed)
  - `inventory.finalized` (Updates status to Shipped)
- **Order Events:**
  - `order.delivered` (Updates status to Delivered)
- **Logistics Events:**
  - `logistics.shipment_created` (Updates status to Confirmed)
  - `logistics.shipment_cancelled` (Updates status to Cancelled)
  - `logistics.shipment_updated` (Updates status to Shipped, Delivered, or Cancelled based on payload status)

---

### F. Logistics Service (Port 3008)

## 1. `GET /health`
### Description
Health check endpoint for the logistics service.

### cURL Command
```bash
curl -X GET http://localhost:3008/health \
  -H "Accept: application/json"
```

### JSON Response
```json
{
  "status": "ok",
  "service": "logistics"
}
```

### Event Mesh Triggers
- **Publishes:** None
- **Subscribes:** None


## 2. `GET /metrics`
### Description
Prometheus metrics endpoint.

### cURL Command
```bash
curl -X GET http://localhost:3008/metrics \
  -H "Accept: text/plain"
```

### JSON Response
*(Note: Returns standard Prometheus text format, not JSON)*

### Event Mesh Triggers
- **Publishes:** None
- **Subscribes:** None


## 3. `POST /shipments`
### Description
Creates a shipment.

### cURL Command
```bash
curl -X POST http://localhost:3008/shipments \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{
    "order_id": "123e4567-e89b-12d3-a456-426614174000",
    "user_id": "223e4567-e89b-12d3-a456-426614174001",
    "supplier_id": "323e4567-e89b-12d3-a456-426614174002",
    "product_id": "423e4567-e89b-12d3-a456-426614174003",
    "notes": "Fragile: Handle with care"
  }'
```

### JSON Response
```json
{
  "id": "523e4567-e89b-12d3-a456-426614174004",
  "order_id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "223e4567-e89b-12d3-a456-426614174001",
  "supplier_id": "323e4567-e89b-12d3-a456-426614174002",
  "product_id": "423e4567-e89b-12d3-a456-426614174003",
  "tracking_number": "TRK-1234567890",
  "status": "pending",
  "notes": "Fragile: Handle with care",
  "created_at": "2026-07-24T22:51:00Z",
  "updated_at": "2026-07-24T22:51:00Z",
  "dispatched_at": null,
  "delivered_at": null
}
```

### Event Mesh Triggers
- **Publishes:** `logistics.shipment_created` (to Redis and RabbitMQ)
- **Subscribes:** None


## 4. `GET /shipments/{shipment_id}`
### Description
Returns shipment details by ID.

### cURL Command
```bash
curl -X GET http://localhost:3008/shipments/523e4567-e89b-12d3-a456-426614174004 \
  -H "Accept: application/json"
```

### JSON Response
```json
{
  "id": "523e4567-e89b-12d3-a456-426614174004",
  "order_id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "223e4567-e89b-12d3-a456-426614174001",
  "supplier_id": "323e4567-e89b-12d3-a456-426614174002",
  "product_id": "423e4567-e89b-12d3-a456-426614174003",
  "tracking_number": "TRK-1234567890",
  "status": "pending",
  "notes": "Fragile: Handle with care",
  "created_at": "2026-07-24T22:51:00Z",
  "updated_at": "2026-07-24T22:51:00Z",
  "dispatched_at": null,
  "delivered_at": null
}
```

### Event Mesh Triggers
- **Publishes:** None
- **Subscribes:** None


## 5. `GET /shipments/supplier/{supplier_id}`
### Description
Returns supplier shipments using filter and pagination query fields.

### cURL Command
```bash
curl -X GET "http://localhost:3008/shipments/supplier/323e4567-e89b-12d3-a456-426614174002?status=intransit&limit=10&offset=0" \
  -H "Accept: application/json"
```

### JSON Response
```json
[
  {
    "id": "523e4567-e89b-12d3-a456-426614174004",
    "order_id": "123e4567-e89b-12d3-a456-426614174000",
    "user_id": "223e4567-e89b-12d3-a456-426614174001",
    "supplier_id": "323e4567-e89b-12d3-a456-426614174002",
    "product_id": "423e4567-e89b-12d3-a456-426614174003",
    "tracking_number": "TRK-1234567890",
    "status": "intransit",
    "notes": "Fragile: Handle with care",
    "created_at": "2026-07-24T22:51:00Z",
    "updated_at": "2026-07-24T22:55:00Z",
    "dispatched_at": "2026-07-24T22:55:00Z",
    "delivered_at": null
  }
]
```

### Event Mesh Triggers
- **Publishes:** None
- **Subscribes:** None


## 6. `PUT /shipments/{shipment_id}/status`
### Description
Updates shipment status and publishes logistics.shipment_updated.

### cURL Command
```bash
curl -X PUT http://localhost:3008/shipments/523e4567-e89b-12d3-a456-426614174004/status \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{
    "status": "delivered",
    "notes": "Left at front door"
  }'
```

### JSON Response
```json
{
  "id": "523e4567-e89b-12d3-a456-426614174004",
  "order_id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "223e4567-e89b-12d3-a456-426614174001",
  "supplier_id": "323e4567-e89b-12d3-a456-426614174002",
  "product_id": "423e4567-e89b-12d3-a456-426614174003",
  "tracking_number": "TRK-1234567890",
  "status": "delivered",
  "notes": "Left at front door",
  "created_at": "2026-07-24T22:51:00Z",
  "updated_at": "2026-07-24T22:58:00Z",
  "dispatched_at": "2026-07-24T22:55:00Z",
  "delivered_at": "2026-07-24T22:58:00Z"
}
```

### Event Mesh Triggers
- **Publishes:** `logistics.shipment_updated` (to Redis and RabbitMQ)
- **Subscribes:** None


## 7. `PUT /shipments/order/{order_id}/cancel`
### Description
Cancels an active shipment by order id and publishes logistics.shipment_cancelled.

### cURL Command
```bash
curl -X PUT http://localhost:3008/shipments/order/123e4567-e89b-12d3-a456-426614174000/cancel \
  -H "Accept: application/json"
```

### JSON Response
```json
{
  "id": "523e4567-e89b-12d3-a456-426614174004",
  "order_id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "223e4567-e89b-12d3-a456-426614174001",
  "supplier_id": "323e4567-e89b-12d3-a456-426614174002",
  "product_id": "423e4567-e89b-12d3-a456-426614174003",
  "tracking_number": "TRK-1234567890",
  "status": "cancelled",
  "notes": "Order cancelled by user",
  "created_at": "2026-07-24T22:51:00Z",
  "updated_at": "2026-07-24T23:00:00Z",
  "dispatched_at": null,
  "delivered_at": null
}
```

### Event Mesh Triggers
- **Publishes:** `logistics.shipment_cancelled` (to Redis and RabbitMQ)
- **Subscribes:** None


## Background Subscriptions (Redis Streams)
Independent of the HTTP routes, the logistics service runs a background task that subscribes to the `"logistics"` consumer group for the following Redis Stream events:
- **`inventory.finalized`**: Subscribes and triggers the creation of a new shipment (publishes `logistics.shipment_created`).
- **`order.cancelled`**: Subscribes and triggers the cancellation of an existing shipment (publishes `logistics.shipment_cancelled`).

---

### G. Payments Service (Port 3009)

## Subscriptions (Event Mesh)
The Payments service subscribes to the `payments` stream for the following events:
- `inventory.reserved`: Auto-generates a PaymentIntent.
- `order.cancelled`: Cancels or refunds a PaymentIntent.
- `order.refunded`: Refunds a PaymentIntent.
- `order.delivered`: Transfers funds to the supplier (Stripe Connect).

## Routes

### 1. Health Check
**Endpoint**: `GET /health`
**cURL**:
```bash
curl -X GET http://localhost:3010/health \
  -H "Accept: application/json"
```
**Expected Response**:
```json
{
  "status": "ok",
  "service": "payments"
}
```
**Event Mesh Triggers**: None

### 2. Metrics
**Endpoint**: `GET /metrics`
**cURL**:
```bash
curl -X GET http://localhost:3010/metrics
```
**Expected Response**: Prometheus metrics (plain text)
**Event Mesh Triggers**: None

### 3. Create Payment Intent
**Endpoint**: `POST /payments/intents`
**cURL**:
```bash
curl -X POST http://localhost:3010/payments/intents \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{
    "idempotency_key": "idemp-12345",
    "order_id": "123e4567-e89b-12d3-a456-426614174000",
    "user_id": "123e4567-e89b-12d3-a456-426614174001",
    "supplier_id": "123e4567-e89b-12d3-a456-426614174002",
    "product_id": "123e4567-e89b-12d3-a456-426614174003",
    "quantity": 2,
    "amount": 5000,
    "currency": "usd",
    "provider": "stripe",
    "metadata": {
      "customer_email": "customer@example.com"
    }
  }'
```
**Expected Response**:
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174010",
  "idempotency_key": "idemp-12345",
  "order_id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "123e4567-e89b-12d3-a456-426614174001",
  "supplier_id": "123e4567-e89b-12d3-a456-426614174002",
  "product_id": "123e4567-e89b-12d3-a456-426614174003",
  "quantity": 2,
  "amount": 5000,
  "currency": "usd",
  "provider": "stripe",
  "provider_reference": "pi_1234567890",
  "status": "requires_payment_method",
  "metadata": {
    "customer_email": "customer@example.com",
    "client_secret": "pi_1234567890_secret_0987654321",
    "stripe_id": "pi_1234567890"
  },
  "created_at": "2026-07-24T22:51:05Z",
  "updated_at": "2026-07-24T22:51:05Z"
}
```
**Event Mesh Triggers**: Publishes `payment.initiated`

### 4. Get Payment Intent
**Endpoint**: `GET /payments/intents/{id}`
**cURL**:
```bash
curl -X GET http://localhost:3010/payments/intents/123e4567-e89b-12d3-a456-426614174010 \
  -H "Accept: application/json"
```
**Expected Response**:
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174010",
  "idempotency_key": "idemp-12345",
  "order_id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "123e4567-e89b-12d3-a456-426614174001",
  "supplier_id": "123e4567-e89b-12d3-a456-426614174002",
  "product_id": "123e4567-e89b-12d3-a456-426614174003",
  "quantity": 2,
  "amount": 5000,
  "currency": "usd",
  "provider": "stripe",
  "provider_reference": "pi_1234567890",
  "status": "requires_payment_method",
  "metadata": {
    "customer_email": "customer@example.com",
    "client_secret": "pi_1234567890_secret_0987654321",
    "stripe_id": "pi_1234567890"
  },
  "created_at": "2026-07-24T22:51:05Z",
  "updated_at": "2026-07-24T22:51:05Z"
}
```
**Event Mesh Triggers**: None

### 5. Mark Payment Succeeded
**Endpoint**: `POST /payments/intents/{id}/succeed`
**cURL**:
```bash
curl -X POST http://localhost:3010/payments/intents/123e4567-e89b-12d3-a456-426614174010/succeed \
  -H "Accept: application/json"
```
**Expected Response**: Same JSON structure as Create Payment Intent, but with `"status": "succeeded"`.
**Event Mesh Triggers**: Publishes `payment.success`

### 6. Mark Payment Failed
**Endpoint**: `POST /payments/intents/{id}/fail`
**cURL**:
```bash
curl -X POST http://localhost:3010/payments/intents/123e4567-e89b-12d3-a456-426614174010/fail \
  -H "Accept: application/json"
```
**Expected Response**: Same JSON structure as Create Payment Intent, but with `"status": "failed"`.
**Event Mesh Triggers**: Publishes `payment.failed`

### 7. Refund Payment
**Endpoint**: `POST /payments/intents/{id}/refund`
**cURL**:
```bash
curl -X POST http://localhost:3010/payments/intents/123e4567-e89b-12d3-a456-426614174010/refund \
  -H "Accept: application/json"
```
**Expected Response**: Same JSON structure as Create Payment Intent, but with `"status": "refunded"`.
**Event Mesh Triggers**: Publishes `payment.refunded`

### 8. Transfer Payment
**Endpoint**: `POST /payments/intents/{id}/transfer`
**cURL**:
```bash
curl -X POST http://localhost:3010/payments/intents/123e4567-e89b-12d3-a456-426614174010/transfer \
  -H "Accept: application/json"
```
**Expected Response**:
```json
{
  "transfer_id": "tr_1234567890",
  "payout_amount_cents": 4750
}
```
**Event Mesh Triggers**: None

### 9. Payment Webhooks (Stripe)
**Endpoint**: `POST /payments/webhooks`
**cURL**:
```bash
curl -X POST http://localhost:3010/payments/webhooks \
  -H "Stripe-Signature: t=12345,v1=signature" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{
    "provider_reference": "pi_1234567890",
    "idempotency_key": "idemp-12345",
    "status": "succeeded",
    "metadata": {
      "source": "stripe_webhook"
    }
  }'
```
**Expected Response**: Same JSON structure as Create Payment Intent, updated with the webhook data (e.g., `"status": "succeeded"`).
**Event Mesh Triggers**: Publishes an event corresponding to the status (e.g., `payment.success` for `succeeded`, `payment.failed` for `failed`, etc.).

---

### H. Notifications Service (Port 3010)

## Event Mesh Triggers

### Subscriptions (Redis Stream: `notifications`)
The service consumes the following events from Redis:
- `order.created`
- `order.cancelled`
- `inventory.lowstock`
- `inventory.rejected`
- `logistics.shipment_created`
- `logistics.shipment_updated`
- `logistics.shipment_cancelled`
- `payment.failed`
- `payment.success`
- `payment.cancelled`
- `supplier.created`
- `supplier.status_updated`
- `user.created`

### Publications (RabbitMQ: `notifications_dlx`)
The service publishes to a Dead Letter Queue (DLQ) upon failed deliveries:
- **Exchange:** `notifications_dlx`
- **Routing Key:** `retry`
- **Queue:** `notifications_retry_queue`

---

## REST API Routes

### 1. Health Check
`GET /health`

**cURL Request:**
```bash
curl -X GET http://localhost:3009/health \
  -H "Accept: application/json"
```

**Expected Response:**
```json
{
  "status": "ok",
  "service": "notifications"
}
```

### 2. Metrics
`GET /metrics`

**cURL Request:**
```bash
curl -X GET http://localhost:3009/metrics \
  -H "Accept: text/plain"
```

**Expected Response:**
*(Prometheus metrics plaintext)*

### 3. Create Notification
`POST /notifications`

**cURL Request:**
```bash
curl -X POST http://localhost:3009/notifications \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{
    "user_id": "123e4567-e89b-12d3-a456-426614174001",
    "supplier_id": "123e4567-e89b-12d3-a456-426614174002",
    "order_id": "123e4567-e89b-12d3-a456-426614174003",
    "event_type": "string",
    "channel": "email",
    "priority": "low",
    "recipient": "string",
    "subject": "string",
    "body": "string",
    "payload": {
      "key": "value"
    }
  }'
```

**Expected Response (201 Created or 202 Accepted):**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "123e4567-e89b-12d3-a456-426614174001",
  "supplier_id": "123e4567-e89b-12d3-a456-426614174002",
  "order_id": "123e4567-e89b-12d3-a456-426614174003",
  "event_type": "string",
  "channel": "email",
  "priority": "low",
  "recipient": "string",
  "subject": "string",
  "body": "string",
  "payload": {
    "key": "value"
  },
  "status": "pending",
  "attempts": 0,
  "last_error": "string",
  "sent_at": "2026-07-24T22:51:05Z",
  "read_at": "2026-07-24T22:51:05Z",
  "created_at": "2026-07-24T22:51:05Z",
  "updated_at": "2026-07-24T22:51:05Z"
}
```

### 4. List Notifications
`GET /notifications`

**cURL Request:**
```bash
curl -X GET "http://localhost:3009/notifications?user_id=123e4567-e89b-12d3-a456-426614174001&supplier_id=123e4567-e89b-12d3-a456-426614174002&status=pending&limit=10&offset=0" \
  -H "Accept: application/json"
```

**Expected Response (200 OK):**
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "user_id": "123e4567-e89b-12d3-a456-426614174001",
    "supplier_id": "123e4567-e89b-12d3-a456-426614174002",
    "order_id": "123e4567-e89b-12d3-a456-426614174003",
    "event_type": "string",
    "channel": "email",
    "priority": "low",
    "recipient": "string",
    "subject": "string",
    "body": "string",
    "payload": {
      "key": "value"
    },
    "status": "pending",
    "attempts": 0,
    "last_error": "string",
    "sent_at": "2026-07-24T22:51:05Z",
    "read_at": "2026-07-24T22:51:05Z",
    "created_at": "2026-07-24T22:51:05Z",
    "updated_at": "2026-07-24T22:51:05Z"
  }
]
```

### 5. Get Notification
`GET /notifications/{id}`

**cURL Request:**
```bash
curl -X GET http://localhost:3009/notifications/123e4567-e89b-12d3-a456-426614174000 \
  -H "Accept: application/json"
```

**Expected Response (200 OK):**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "123e4567-e89b-12d3-a456-426614174001",
  "supplier_id": "123e4567-e89b-12d3-a456-426614174002",
  "order_id": "123e4567-e89b-12d3-a456-426614174003",
  "event_type": "string",
  "channel": "email",
  "priority": "low",
  "recipient": "string",
  "subject": "string",
  "body": "string",
  "payload": {
    "key": "value"
  },
  "status": "pending",
  "attempts": 0,
  "last_error": "string",
  "sent_at": "2026-07-24T22:51:05Z",
  "read_at": "2026-07-24T22:51:05Z",
  "created_at": "2026-07-24T22:51:05Z",
  "updated_at": "2026-07-24T22:51:05Z"
}
```

### 6. Mark Notification Read
`PUT /notifications/{id}/read`

**cURL Request:**
```bash
curl -X PUT http://localhost:3009/notifications/123e4567-e89b-12d3-a456-426614174000/read \
  -H "Accept: application/json"
```

**Expected Response (200 OK):**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "123e4567-e89b-12d3-a456-426614174001",
  "supplier_id": "123e4567-e89b-12d3-a456-426614174002",
  "order_id": "123e4567-e89b-12d3-a456-426614174003",
  "event_type": "string",
  "channel": "email",
  "priority": "low",
  "recipient": "string",
  "subject": "string",
  "body": "string",
  "payload": {
    "key": "value"
  },
  "status": "read",
  "attempts": 0,
  "last_error": "string",
  "sent_at": "2026-07-24T22:51:05Z",
  "read_at": "2026-07-24T22:51:05Z",
  "created_at": "2026-07-24T22:51:05Z",
  "updated_at": "2026-07-24T22:51:05Z"
}
```

### 7. Register Device
`POST /notification-devices`

**cURL Request:**
```bash
curl -X POST http://localhost:3009/notification-devices \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{
    "user_id": "123e4567-e89b-12d3-a456-426614174001",
    "platform": "ios",
    "push_token": "string",
    "provider": "string",
    "device_id": "string",
    "app_version": "string"
  }'
```

**Expected Response (201 Created):**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "123e4567-e89b-12d3-a456-426614174001",
  "platform": "ios",
  "push_token": "string",
  "provider": "string",
  "device_id": "string",
  "app_version": "string",
  "enabled": true,
  "last_seen_at": "2026-07-24T22:51:05Z",
  "created_at": "2026-07-24T22:51:05Z",
  "updated_at": "2026-07-24T22:51:05Z"
}
```

### 8. List User Devices
`GET /notification-devices/user/{user_id}`

**cURL Request:**
```bash
curl -X GET http://localhost:3009/notification-devices/user/123e4567-e89b-12d3-a456-426614174001 \
  -H "Accept: application/json"
```

**Expected Response (200 OK):**
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "user_id": "123e4567-e89b-12d3-a456-426614174001",
    "platform": "ios",
    "push_token": "string",
    "provider": "string",
    "device_id": "string",
    "app_version": "string",
    "enabled": true,
    "last_seen_at": "2026-07-24T22:51:05Z",
    "created_at": "2026-07-24T22:51:05Z",
    "updated_at": "2026-07-24T22:51:05Z"
  }
]
```

### 9. Disable Device
`DELETE /notification-devices/{id}`

**cURL Request:**
```bash
curl -X DELETE http://localhost:3009/notification-devices/123e4567-e89b-12d3-a456-426614174000 \
  -H "Accept: application/json"
```

**Expected Response (200 OK):**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "123e4567-e89b-12d3-a456-426614174001",
  "platform": "ios",
  "push_token": "string",
  "provider": "string",
  "device_id": "string",
  "app_version": "string",
  "enabled": false,
  "last_seen_at": "2026-07-24T22:51:05Z",
  "created_at": "2026-07-24T22:51:05Z",
  "updated_at": "2026-07-24T22:51:05Z"
}
```

### 10. Get Preferences
`GET /notification-preferences/user/{user_id}`

**cURL Request:**
```bash
curl -X GET http://localhost:3009/notification-preferences/user/123e4567-e89b-12d3-a456-426614174001 \
  -H "Accept: application/json"
```

**Expected Response (200 OK):**
```json
{
  "user_id": "123e4567-e89b-12d3-a456-426614174001",
  "email_enabled": true,
  "sms_enabled": true,
  "push_enabled": true,
  "in_app_enabled": true,
  "created_at": "2026-07-24T22:51:05Z",
  "updated_at": "2026-07-24T22:51:05Z"
}
```

### 11. Update Preferences
`PUT /notification-preferences/user/{user_id}`

**cURL Request:**
```bash
curl -X PUT http://localhost:3009/notification-preferences/user/123e4567-e89b-12d3-a456-426614174001 \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{
    "email_enabled": true,
    "sms_enabled": true,
    "push_enabled": true,
    "in_app_enabled": true
  }'
```

**Expected Response (200 OK):**
```json
{
  "user_id": "123e4567-e89b-12d3-a456-426614174001",
  "email_enabled": true,
  "sms_enabled": true,
  "push_enabled": true,
  "in_app_enabled": true,
  "created_at": "2026-07-24T22:51:05Z",
  "updated_at": "2026-07-24T22:51:05Z"
}
```

---

### I. Analytics Service (Port 3007)

## Routes

### POST `/analytics`
Retrieves aggregated analytics metrics from the database based on event data. This route accepts both query parameters and a JSON body. The body takes precedence over query parameters if both are provided.

#### Curl Command
```bash
curl -X POST "http://localhost:3007/analytics?metric=signups&window=30d&group_by=country&aggregate_field=signups&limit=10&order_by=value_desc&signup_source=web" \
  -H "Content-Type: application/json" \
  -d '{
    "metric": "signups",
    "window": "30d",
    "group_by": "country",
    "aggregate_field": "signups",
    "limit": 10,
    "order_by": "value_desc",
    "filters": {
      "country": "US",
      "signup_source": "web"
    }
  }'
```

#### JSON Expected Response
The service returns the generated SQL query along with the raw resulting rows in JSON format.
```json
{
  "sql": "SELECT COALESCE(json_agg(t), '[]'::json) AS data FROM ( \n                    SELECT country, SUM(signups)::numeric AS value\n                    FROM analytics.user_signups_daily\n                    WHERE day >= NOW() - INTERVAL '30 days' AND ( (country = $1) OR ((data->>'country') = $2) ) AND ( (signup_source = $3) OR ((data->>'signup_source') = $4) )\n                    GROUP BY country\n                    ORDER BY value DESC LIMIT 10\n                  ) t",
  "result": [
    {
      "country": "US",
      "value": 152
    },
    {
      "country": "UK",
      "value": 43
    }
  ]
}
```

## Event Mesh Triggers

### Subscriptions
- **Exchange:** `analytics_events_topic`
- **Routing Key:** `#` (Listens to all topics/events)
- **Queue:** `analytics_queue`
- **Handling:** Processes arbitrary analytics events, standardizes them into the `Event` schema (containing an inner `AnalyticsEvent`), and writes them to PostgreSQL and Redis. Supported implicit event types include `product.viewed`, `order.created`, `user.created`, etc. 

### Publications
- **Exchange:** None directly for business logic.
- **DLQ Routing:** Unprocessable messages (invalid format or failed insertion after 3 retries) are sent to the `analytics_dlq` dead-letter queue.
