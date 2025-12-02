how analytics will be written into their schema, 
NOTE: this content is AI generated

```
event_type = 'x.y'
event_timestamp = timestamptz
data = {...}
```

---

# ✅ **GLOBAL EVENTS TABLE (Raw Event Store)**

This is your base ingestion table.

```sql
CREATE TABLE analytics.events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    event_timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    data JSONB NOT NULL
);

SELECT create_hypertable('analytics.events', 'event_timestamp');
```

---

# =====================================================================

# 🧩 **1. USER EVENTS**

# =====================================================================

## **1.1 USER SIGN-UP – `user.created`**

### **Schema**

```sql
-- event_type = 'user.created'
-- event_timestamp = timestamptz
-- data = {
--     "user_id": "uuid",
--     "source": "web" | "mobile" | "referral" | "campaign" | "ads" | "unknown",
--     "country": "NG" | "US" | "KE" | ...,
--     "referrer_id": "uuid | null",
--     "platform": "ios" | "android" | "web"
-- }
```

### **Sample Event**

```sql
INSERT INTO analytics.events (event_type, data) VALUES (
   'user.created',
   '{
       "user_id": "5678-1234-2233",
       "source": "web",
       "country": "NG",
       "referrer_id": null,
       "platform": "web"
   }'::jsonb
);
```

### **Analytics Query: Signups per day**

```sql
SELECT time_bucket('1 day', event_timestamp) AS day,
       COUNT(*) AS signups
FROM analytics.events
WHERE event_type = 'user.created'
GROUP BY day
ORDER BY day DESC;
```

---

## **1.2 USER LOGIN – `user.logged_in`**

### **Schema**

```sql
-- event_type = 'user.logged_in'
-- data = {
--    "user_id": "uuid",
--    "platform": "ios" | "android" | "web",
--    "method": "password" | "google" | "github"
-- }
```

### **Sample Event**

```sql
INSERT INTO analytics.events (event_type, data) VALUES (
  'user.logged_in',
  '{
      "user_id": "5678-1234-2233",
      "platform": "android",
      "method": "google"
  }'::jsonb
);
```

### **Analytics Query: DAU**

```sql
SELECT time_bucket('1 day', event_timestamp) AS day,
       COUNT(DISTINCT data->>'user_id') AS dau
FROM analytics.events
WHERE event_type = 'user.logged_in'
GROUP BY day
ORDER BY day DESC;
```

---

# =====================================================================

# 🛒 **2. PRODUCT & VIEW EVENTS**

# =====================================================================

## **2.1 PRODUCT VIEW – `product.viewed`**

### **Schema**

```sql
-- event_type = 'product.viewed'
-- data = {
--     "user_id": "uuid | null",
--     "product_id": "uuid",
--     "source": "search" | "home" | "category" | "external",
--     "device": "mobile" | "desktop"
-- }
```

### **Sample Event**

```sql
INSERT INTO analytics.events (event_type, data) VALUES (
  'product.viewed',
  '{
      "user_id": null,
      "product_id": "p123",
      "source": "search",
      "device": "mobile"
  }'::jsonb
);
```

### **Analytics Query: Views per product (7d)**

```sql
SELECT data->>'product_id' AS product_id,
       COUNT(*) AS views_7d
FROM analytics.events
WHERE event_type = 'product.viewed'
  AND event_timestamp >= NOW() - interval '7 days'
GROUP BY product_id
ORDER BY views_7d DESC;
```

---

# =====================================================================

# 🛍️ **3.ORDER EVENTS**

# =====================================================================

## **3.1 CART ADD – `order.created`**

### **Schema**

```sql
-- event_type = 'order.created'
-- data = {
--     "order_id": "uuid",
--     "product_id": "uuid",
--     "qty": 1,
--     "price": 2500
-- }
```

### **Sample Event**

```sql
INSERT INTO analytics.events (event_type, data) VALUES (
  'cart.item_added',
  '{
      "user_id": "u100",
      "product_id": "p200",
      "qty": 2,
      "price": 2500
  }'::jsonb
);
```

<!-- for daily revenue 
-- SELECT *
-- FROM analytics.revenue_daily
-- WHERE day >= NOW() - interval '30 days'
-- ORDER BY day DESC; -->

### **Analytics Query: Cart Additions**

```sql
SELECT data->>'product_id' AS product_id,
       SUM((data->>'qty')::int) AS total_added
FROM analytics.events
WHERE event_type = 'cart.item_added'
GROUP BY product_id
ORDER BY total_added DESC;
```

---

# =====================================================================

# 💳 **4. ORDER EVENTS**

# =====================================================================

## **4.1 ORDER PLACED – `order.created`**

### **Schema**

```sql
-- event_type = 'order.created'
-- data = {
--   "order_id": "uuid",
--   "user_id": "uuid",
--   "amount": 5500,
--   "items": [
--        {"product_id": "p100", "qty": 2, "price": 2000},
--        {"product_id": "p200", "qty": 1, "price": 1500}
--   ],
--   "currency": "NGN"
-- }
```

### **Sample Event**

```sql
INSERT INTO analytics.events (event_type, data) VALUES (
  'order.created',
  '{
      "order_id": "ord123",
      "user_id": "u100",
      "amount": 5500,
      "currency": "NGN",
      "items": [
         {"product_id": "p100", "qty": 2, "price": 2000},
         {"product_id": "p200", "qty": 1, "price": 1500}
      ]
  }'::jsonb
);
```

### **Analytics Query: Revenue by day**

```sql
SELECT time_bucket('1 day', event_timestamp) AS day,
       SUM((data->>'amount')::int) AS revenue
FROM analytics.events
WHERE event_type = 'order.created'
GROUP BY day
ORDER BY day DESC;
```

---

# =====================================================================

# ⭐ **5. REVIEW EVENTS**

# =====================================================================

## **5.1 PRODUCT REVIEW – `product.reviewed`**

### **Schema**

```sql
-- event_type = 'product.reviewed'
-- data = {
--     "review_id": "uuid",
--     "user_id": "uuid",
--     "product_id": "uuid",
--     "rating": 1..5,
--     "text": "string"
-- }
```

### **Sample Event**

```sql
INSERT INTO analytics.events (event_type, data) VALUES (
  'product.reviewed',
  '{
      "review_id": "rev789",
      "user_id": "u100",
      "product_id": "p100",
      "rating": 5,
      "text": "Amazing!"
  }'::jsonb
);
```

### **Analytics Query: Average rating**

```sql
SELECT data->>'product_id' AS product,
       AVG((data->>'rating')::float) AS avg_rating
FROM analytics.events
WHERE event_type = 'product.reviewed'
GROUP BY product;
```

---

# =====================================================================

# ⏳ **ROLLUP TABLE STRUCTURES (Materialized Tables)**

# =====================================================================

These are optimized reporting tables.

---

## **A. product daily views**

```sql
CREATE TABLE analytics.product_views_daily (
  product_id TEXT,
  bucket TIMESTAMPTZ,
  views INT
);

SELECT create_hypertable('analytics.product_views_daily', 'bucket');
```

### **Query to populate**

```sql
INSERT INTO analytics.product_views_daily (product_id, bucket, views)
SELECT data->>'product_id',
       time_bucket('1 day', event_timestamp),
       COUNT(*)
FROM analytics.events
WHERE event_type = 'product.viewed'
GROUP BY 1,2;
```

---

## **B. top_products_7d (example you referenced)**

```sql
CREATE MATERIALIZED VIEW analytics.top_products_7d AS
SELECT 
  data->>'product_id' AS product_id,
  time_bucket('1 day', event_timestamp) AS bucket,
  COUNT(*) AS views,
  0 AS sold_qty  -- updated by order rollup
FROM analytics.events
WHERE event_type = 'product.viewed'
GROUP BY 1,2;
```

### **Query (THIS IS THE ONE YOU REFERENCED):**

```sql
-- SELECT product_id,
--        sum(sold_qty) AS sold_last_7d,
--        sum(views) AS views_last_7d
-- FROM analytics.top_products_7d
-- WHERE bucket >= NOW() - interval '7 days'
-- GROUP BY product_id
-- ORDER BY sold_last_7d DESC
-- LIMIT 100;
```

---

# =====================================================================

# 🧩 **FINAL: All Event Types in One List**

Here is the consolidated event taxonomy:

```
user.created
user.logged_in
product.viewed
cart.item_added
order.created
product.reviewed
```

Every event includes:

```
id (uuid)
event_type (text)
event_timestamp (timestamptz)
data (jsonb)
```





-- aggregated table for daily event counts
CREATE TABLE IF NOT EXISTS daily_event_counts (
    event_date DATE NOT NULL,
    event_type TEXT NOT NULL,
    count BIGINT NOT NULL,
    PRIMARY KEY (event_date, event_type)
);

CREATE INDEX IF NOT EXISTS idx_daily_event_counts_event_date
ON daily_event_counts (event_date);

CREATE TABLE IF NOT EXISTS monthly_event_counts (
    event_month DATE NOT NULL,
    event_type TEXT NOT NULL,
    count BIGINT NOT NULL,
    PRIMARY KEY (event_month, event_type)
);

CREATE TABLE IF NOT EXISTS annual_event_counts (
    event_year DATE NOT NULL,
    event_type TEXT NOT NULL,
    count BIGINT NOT NULL,
    PRIMARY KEY (event_year, event_type)
);



-- aggregated table for product views
CREATE TABLE IF NOT EXISTS product_views_daily (
    day DATE NOT NULL,
    product_id UUID NOT NULL,
    view_count BIGINT NOT NULL,
    PRIMARY KEY (day, product_id)
)

CREATE INDEX IF NOT EXISTS idx_product_views_daily_day
ON product_views_daily (day);

CREATE TABLE IF NOT EXISTS product_views_monthly (
    month DATE NOT NULL,
    product_id UUID NOT NULL,
    view_count BIGINT NOT NULL,
    PRIMARY KEY (month, product_id)
);

CREATE TABLE IF NOT EXISTS product_views_annual (
    year DATE NOT NULL,
    product_id UUID NOT NULL,
    view_count BIGINT NOT NULL,
    PRIMARY KEY (year, product_id)
);



-- aggregated table for user signups
CREATE TABLE IF NOT EXISTS user_signups_daily (
    day DATE NOT NULL,
    signup_count BIGINT NOT NULL,
    user_id UUID NOT NULL,
    PRIMARY KEY (day, user_id)
);

CREATE INDEX IF NOT EXISTS idx_user_signups_daily_day
ON user_signups_daily (day);

CREATE TABLE IF NOT EXISTS user_signups_monthly (
    month DATE NOT NULL,
    signup_count BIGINT NOT NULL,
    user_id UUID NOT NULL,
    PRIMARY KEY (month, user_id)
);

CREATE TABLE IF NOT EXISTS user_signups_annual (
    year DATE NOT NULL,
    signup_count BIGINT NOT NULL,
    user_id UUID NOT NULL,
    PRIMARY KEY (year, user_id)
);



-- aggregated table for places orders views
CREATE TABLE IF NOT EXISTS orders_daily (
    day DATE NOT NULL,
    order_count BIGINT NOT NULL,
    order_id UUID NOT NULL,
    revenue NUMERIC(20, 2) NOT NULL,
    PRIMARY KEY (day)
);

CREATE INDEX IF NOT EXISTS idx_orders_daily_day
ON orders_daily (day);

CREATE TABLE IF NOT EXISTS orders_monthly (
    month DATE NOT NULL,
    order_count BIGINT NOT NULL,
    order_id UUID NOT NULL,
    revenue NUMERIC(20, 2) NOT NULL,
    PRIMARY KEY (month)
);

CREATE TABLE IF NOT EXISTS orders_annual (
    year DATE NOT NULL,
    order_count BIGINT NOT NULL,
    order_id UUID NOT NULL,
    revenue NUMERIC(20, 2) NOT NULL,
    PRIMARY KEY (year)
);