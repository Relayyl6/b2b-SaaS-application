# 📦 Order Service — README

The **Order Service** manages customer orders within the distributed commerce platform.
It handles creating orders, retrieving order information, updating order status, and deleting orders.
The service also publishes domain events to **Redis Streams** (RabbitMQ optional) so other services such as **Inventory**, **Payments**, **Notifications**, and **Logistics** can react.

---

## 🧱 **Tech Stack**

* **Rust**
* **Actix-Web**
* **SQLx (PostgreSQL)**
* **Redis (Pub/Sub + Stream publishing via RedisPublisher)**
* **UUID**
* **Chrono**

---

## 📁 Folder Structure (Typical)

```
/src
  ├── models.rs
  ├── redis_pub.rs
  ├── routes
  │     └── orders.rs
  ├── main.rs
```

---

## 📡 API Endpoints

### 1️⃣ **Create an Order**

`POST /orders`

Creates a new order and publishes an `order.created` event.

#### Request Body (CreateOrderRequest)

```json
{
  "user_id": "uuid",
  "supplier_id": "uuid",
  "product_id": "uuid",
  "qty": 3,
  "status": "Pending",
  "items": {
    "color": "black",
    "size": "L"
  }
}
```

#### Recommended request body (with full implementation beyond simple request)

```json
{
  "user_id": "UUID",
  "supplier_id": "UUID",
  "product_id": "UUID",
  "qty": 1,
  "status": null, 
  "items": {
    "name": "string",
    "unit_price": 0,
    "quantity": 0,
    "currency": "string",
    "category": "string",
    "sku": "string",

    "subtotal": 0,

    "discount": {
      "type": "none | voucher | bulk | wholesale | seasonal",
      "amount": 0
    },

    "specifications": {
      "color": "optional string",
      "size": "optional string",
      "weight": "optional string",
      "volume": "optional string",
      "storage": "optional string",
      "warranty": "optional string",
      "ram": "optional string",
      "material": "optional string",
      "other_specs": {}
    },

    "shipping": {
      "delivery_type": "doorstep | pickup_station",
      "method": "string",
      "station_location": "optional string",
      "estimated_delivery_days": 0,
      "estimated_ready_in_hours": 0,
      "shipping_fee": 0
    },

    "logistics": {
      "requires_heavy_transport": false,
      "truck_type": "optional string",
      "offloading_required": false
    },

    "tax": {
      "vat_percentage": 0,
      "vat_amount": 0
    },

    "final_total": 0,

    "metadata": {
      "notes": "optional string",
      "delivery_instructions": "optional string",
      "gift_wrapping": false
    }
  }
}
```

#### Response

```json
{
  "message": "Order successfully created",
  "id": { ...full order object... }
}
```

#### Events Published

* `order.created`

---

### 2️⃣ **Get Order by ID**

`GET /orders/{order_id}`

Returns a full order object.

#### Example Response

```json
{
  "id": "uuid",
  "user_id": "uuid",
  "supplier_id": "uuid",
  "product_id": "uuid",
  "qty": 5,
  "status": "Pending",
  "expires_at": "2025-11-26T10:00:00Z",
  ...
}
```

---

### 3️⃣ **Update Order Status**

`PUT /orders/{id}/status`

Used by:

* **Inventory Service** → to confirm or reject an order
* **Payment Service** → to confirm payment
* **Logistics** → to mark delivered

#### Request Body (UpdateOrderStatus)

```json
{
  "new_status": "Confirmed",
  "user_id": "uuid",
  "product_id": "uuid",
  "order_timestamp": "2025-11-26T06:16:11Z",
  "expires_at": "2025-12-01T00:00:00Z"
}
```

#### Effects Based on Status

| Status        | System Behaviour                             |
| ------------- | -------------------------------------------- |
| **Confirmed** | order.confirmed logic, notify logistics      |
| **Failed**    | notify user, trigger refund, clean up        |
| **Cancelled** | publish `order.cancelled`, release inventory |
| **Delivered** | mark delivered (soft delete recommended)     |
| **Pending**   | no major effect (sync timers)                |

#### Possible Events Published

* `order.cancelled`

---

### 4️⃣ **Delete Order**

`DELETE /orders/{id}/{user_id}`

Deletes an order *only if it belongs to the user*.

#### Response

```
Order deleted successfully
```

> NOTE: Pending orders should eventually auto-expire → a cron or background job should update them to `Failed` and publish `order.expired`.

---

## 🔌 Event Structure (OrderEvent)

Events published to Redis follow:

```json
{
  "event_type": "order.created",
  "product_id": "uuid",
  "supplier_id": "uuid",
  "order_id": "uuid",
  "quantity": 6,
  "user_id": "uuid",
  "reservation_id": null,
  "expires_at": "2025-11-30T00:00:00Z",
  "order_timestamp": "2025-11-26T06:20:00Z"
}
```

---

## 🗃 Database Schema (Orders Table)

```sql
CREATE TABLE orders (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    supplier_id UUID NOT NULL,
    product_id UUID NOT NULL,
    qty INT NOT NULL,
    status TEXT NOT NULL,
    items JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    order_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

> Consider adding for soft deletes:

```sql
ALTER TABLE orders ADD COLUMN deleted_at TIMESTAMPTZ NULL;
```

---

## 🔄 Order Lifecycle (Event Driven)

```
Client → create_order → (order.created)
        ↓
Inventory Service listens
        ↓
inventory.reserved → update_status(Confirmed)
inventory.rejected → update_status(Failed)

Payments confirms → update_status(Confirmed)
User cancels → update_status(Cancelled)

Logistics → update_status(Delivered)
```

---

## 📝 Example Status Flow

1. Customer places order → `order.created`
2. Inventory reserves product → `order.confirmed`
3. Payment succeeds → `order.confirmed`
4. Supplier ships → logistics workflow
5. Customer receives → `Delivered`
6. Soft delete after X days

---

## 🚀 Running the Service

### Environment Variables

```
DATABASE_URL=postgres://...
REDIS_URL=redis://localhost:6379
```

### Run

```
cargo run
```

---

## 🧪 Testing with cURL

#### Create Order

```bash
curl -X POST http://localhost:8000/orders \
-H "Content-Type: application/json" \
-d '{"user_id":"...", "supplier_id":"...", "product_id":"...", "qty":3, "items":{}}'
```

#### Update Status

```bash
curl -X PUT http://localhost:8000/orders/<order_id>/status \
-H "Content-Type: application/json" \
-d '{"new_status":"Cancelled","user_id":"...","product_id":"..."}'
```

