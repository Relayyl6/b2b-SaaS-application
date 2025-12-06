-- =====================================
-- SECTION: TABLES -- already in a prior migration
-- =====================================

-- =====================================
-- SECTION: MATERIALIZED VIEWS
-- =====================================
-- CREATE MATERIALIZED VIEW analytics.top_products_7d
-- WITH (timescaledb.continuous, timescaledb.refresh_lag = '5 minutes')
-- AS
-- SELECT
--     time_bucket('1 day', event_timestamp) AS bucket,
--     (data->>'product_id')::bigint AS product_id,

--     -- added to cart
--     sum((data->>'quantity')::bigint)
--         FILTER (WHERE event_type = 'order.created') AS carted_qty,
    
--     -- inventory events
--     sum((data->>'quantity')::bigint)
--         FILTER (WHERE event_type = 'inventory.updated') AS restocked_qty,

--     -- logistics
--     count(*)
--         FILTER (WHERE event_type = 'order.shipped') AS shipped_orders,

--     -- products
--     count(*) 
--         FILTER (WHERE event_type = 'product.viewed') AS views,
--     count(*) 
--         FILTER (WHERE event_type = 'product.created') AS created_products,

--     -- payments
--     sum((data->>'amount')::numeric)
--         FILTER (WHERE event_type = 'payment.completed')    AS payment_volume

-- FROM analytics.events
-- WHERE event_type IN ('order.created', 'product.viewed')
-- GROUP BY bucket, product_id;




-- use in consumption
-- SELECT product_id,
--        sum(sold_qty) AS sold_last_7d,
--        sum(views) AS views_last_7d
-- FROM analytics.top_products_7d
-- WHERE bucket >= NOW() - interval '7 days'
-- GROUP BY product_id
-- ORDER BY sold_last_7d DESC
-- LIMIT 100;





-- use in consumption
-- SELECT *
-- FROM analytics.revenue_daily
-- WHERE day >= NOW() - interval '30 days'
-- ORDER BY day DESC;


-- eg user sign up event
-- event_type = 'user.created'
-- event_timestamp = timestamptz
-- data = {
--     "user_id": "...",
--     "source": "web|mobile|referral|campaign|ads|unknown",
--     "country": "NG" / "US" / "KE" / etc,
--     "referrer_id": "...",  -- optional
--     "platform": "ios" | "android" | "web"
-- }

-- use in consumption
--      daily last 30 days signup
-- SELECT *
-- FROM analytics.user_signups_daily
-- WHERE day >= NOW() - INTERVAL '30 days'
-- ORDER BY day DESC;
        -- Top sign up by country(last 7 days)
-- SELECT country, SUM(signups) AS total
-- FROM analytics.user_signups_daily
-- WHERE day >= NOW() - INTERVAL '7 days'
-- GROUP BY country
-- ORDER BY total DESC;
--       Sign up sources in last 7 days
-- SELECT signup_source, SUM(signups) AS total
-- FROM analytics.user_signups_daily
-- WHERE day >= NOW() - INTERVAL '7 days'
-- GROUP BY signup_source
-- ORDER BY total DESC;
-- etc 




-- Basic indexes to support common filters
CREATE INDEX IF NOT EXISTS idx_events_time
    ON analytics.events (event_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_events_type_time
    ON analytics.events (event_type, event_timestamp DESC);

-- Useful partial indexes for common JSONB keys
CREATE INDEX IF NOT EXISTS idx_events_user_id
    ON analytics.events (((data->>'user_id')))
    WHERE event_type = 'user.created';
CREATE INDEX IF NOT EXISTS idx_events_product_id
    ON analytics.events (((data->>'product_id')))
    WHERE (event_type LIKE 'product.%' OR event_type LIKE 'order.%' OR event_type LIKE 'inventory.%');


-- ==========================
-- 2. Common utility: cast helper (SQL function)
-- ==========================
-- Small helper to safely extract numeric from JSONB
CREATE OR REPLACE FUNCTION derived.safe_numeric(j jsonb, key text, default numeric)
RETURNS numeric LANGUAGE sql IMMUTABLE AS $$
    SELECT COALESCE(NULLIF((j->>key)::numeric, NULL), default);
$$;



-- ==========================
-- 3. Users: signups, daily aggregates, cohorts
-- ==========================

-- Indexes specifically for user.created
CREATE INDEX IF NOT EXISTS idx_events_user_created_time
    ON analytics.events (event_timestamp DESC)
    WHERE event_type = 'user.created';
CREATE INDEX IF NOT EXISTS idx_events_user_country
    ON analytics.events ((data->>'country'))
    WHERE event_type = 'user.created';
CREATE INDEX IF NOT EXISTS idx_events_user_source
    ON analytics.events ((data->>'source'))
    WHERE event_type = 'user.created';

-- Continuous aggregate: daily user signups
CREATE MATERIALIZED VIEW IF NOT EXISTS analytics.user_signups_daily
WITH (timescaledb.continuous, timescaledb.refresh_lag = '5 minutes')
AS
SELECT
    time_bucket('1 day', event_timestamp) AS day,
    data->>'source' AS signup_source,
    data->>'platform' AS signup_platform,
    data->>'country' AS country,
    count(*)  AS signups,
    count(DISTINCT (data->>'user_id')) AS unique_users,
    count(*) FILTER (WHERE data->>'referrer_id' IS NOT NULL) AS referred_signups
FROM analytics.events
WHERE event_type = 'user.created'
GROUP BY day, signup_source, signup_platform, country;

-- Add a refresh policy
SELECT add_continuous_aggregate_policy(
    'analytics.user_signups_daily',
    start_offset => INTERVAL '90 days',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '5 minutes'
);











-- ==========================
-- 4. Orders: orders created, shipped, revenue, order size
-- ==========================

CREATE INDEX IF NOT EXISTS idx_events_order_id
    ON analytics.events ((data->>'order_id'))
    WHERE event_type LIKE 'order.%';

-- Orders daily rollup (counts, quantity, revenue)
CREATE MATERIALIZED VIEW IF NOT EXISTS analytics.orders_daily
WITH (timescaledb.continuous, timescaledb.refresh_lag = '5 minutes')
AS
SELECT
    time_bucket('1 day', event_timestamp) AS day,
    (data->>'order_id')::text AS order_id_sample, -- for debugging, not primary
    count(*)
        FILTER (WHERE event_type = 'order.created') AS orders_created,
    sum( (data->>'quantity')::bigint )
        FILTER (WHERE event_type = 'order.created') AS items_ordered,
    sum( (data->>'price')::numeric )
        FILTER (WHERE event_type = 'order.created') AS order_value_created,

    count(*) FILTER (WHERE event_type = 'order.shipped') AS orders_shipped,
    sum( (data->>'price')::numeric ) FILTER (WHERE event_type = 'order.shipped') AS revenue_shipped
FROM analytics.events
WHERE event_type IN ('order.created','order.shipped')
GROUP BY day, order_id_sample;

SELECT add_continuous_aggregate_policy(
    'analytics.orders_daily',
    start_offset => INTERVAL '180 days',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '5 minutes'
);

-- Derived: revenue by day (simpler consumption)
CREATE MATERIALIZED VIEW IF NOT EXISTS analytics.revenue_daily
WITH (timescaledb.continuous)
AS
SELECT
    time_bucket('1 day', event_timestamp) AS day,
    sum((data->>'price')::numeric)
        FILTER (WHERE event_type = 'order.shipped') AS revenue
FROM analytics.events
WHERE event_type = 'order.shipped'
GROUP BY day;

SELECT add_continuous_aggregate_policy(
    'analytics.revenue_daily',
    start_offset => INTERVAL '365 days',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '10 minutes'
);







-- ==========================
-- 5. Products: views, top-sellers, product-level metrics
-- ==========================

CREATE MATERIALIZED VIEW IF NOT EXISTS analytics.product_views_daily
WITH (timescaledb.continuous, timescaledb.refresh_lag = '10 minutes')
AS
SELECT
    time_bucket('1 day', event_timestamp) AS day,
    (data->>'product_id')::bigint AS product_id,
    count(*)
        FILTER (WHERE event_type = 'product.viewed') AS views,
    count(DISTINCT (data->>'session_id'))
        FILTER (WHERE event_type = 'product.viewed') AS unique_sessions,
    count(*) 
        FILTER (WHERE event_type = 'product.created') AS created_products,

FROM analytics.events
WHERE event_type IN ('product.viewed', 'product.created')
GROUP BY day, product_id;

SELECT add_continuous_aggregate_policy(
    'analytics.product_views_daily',
    start_offset => INTERVAL '90 days',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '10 minutes'
);

-- Master product metrics that combine sales + views + inventory
CREATE MATERIALIZED VIEW IF NOT EXISTS analytics.product_metrics_daily
WITH (timescaledb.continuous, timescaledb.refresh_lag = '10 minutes')
AS
SELECT
    time_bucket('1 day', event_timestamp) AS day,
    (data->>'product_id')::bigint AS product_id,

    -- sales
    sum((data->>'quantity')::bigint) FILTER (WHERE event_type = 'order.created') AS sold_qty,
    sum((data->>'price')::numeric) FILTER (WHERE event_type = 'order.created') AS gross_sales,

    -- shipped -> revenue realized
    sum((data->>'price')::numeric) FILTER (WHERE event_type = 'order.shipped') AS revenue_shipped,

    -- views (reuse same events source)
    count(*) FILTER (WHERE event_type = 'product.viewed') AS views,

    -- inventory
    sum((data->>'quantity')::bigint) FILTER (WHERE event_type = 'inventory.updated') AS inventory_delta

FROM analytics.events
WHERE event_type IN ('order.created', 'order.shipped', 'product.viewed', 'inventory.updated')
GROUP BY day, product_id;

SELECT add_continuous_aggregate_policy(
    'analytics.product_metrics_daily',
    start_offset => INTERVAL '365 days',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '15 minutes'
);

-- ==========================
-- 6. Inventory: stock levels and restocks
-- ==========================

CREATE MATERIALIZED VIEW IF NOT EXISTS analytics.inventory_daily
WITH (timescaledb.continuous, timescaledb.refresh_lag = '30 minutes')
AS
SELECT
    time_bucket('1 day', event_timestamp) AS day,
    (data->>'product_id')::bigint AS product_id,
    sum((data->>'quantity')::bigint)
        FILTER (WHERE event_type = 'inventory.updated'
        AND (data->>'change_type') = 'restock') AS restocked_qty,
    sum((data->>'quantity')::bigint)
        FILTER (WHERE event_type = 'inventory.updated'
        AND (data->>'change_type') = 'deduct') AS deducted_qty
FROM analytics.events
WHERE event_type = 'inventory.updated'
GROUP BY day, product_id;

SELECT add_continuous_aggregate_policy(
    'analytics.inventory_daily',
    start_offset => INTERVAL '365 days',
    end_offset => INTERVAL '10 minutes',
    schedule_interval => INTERVAL '30 minutes'
);





-- ==========================
-- 7. Logistics: shipping performance
-- ==========================

CREATE MATERIALIZED VIEW IF NOT EXISTS analytics.delivery_performance_daily
WITH (timescaledb.continuous, timescaledb.refresh_lag = '30 minutes')
AS
SELECT
    time_bucket('1 day', event_timestamp) AS day,
    (data->>'carrier')::text AS carrier,
    count(*)
        FILTER (WHERE event_type = 'order.shipped') AS shipped_count,
    avg((data->>'transit_time_hours')::numeric)
        FILTER (WHERE event_type = 'order.shipped' 
        AND (data->>'transit_time_hours') IS NOT NULL) AS avg_transit_hours,
    percentile_cont(0.95)
        WITHIN GROUP (ORDER BY (data->>'transit_time_hours')::numeric)
        FILTER (WHERE event_type = 'order.shipped' AND (data->>'transit_time_hours') IS NOT NULL) AS p95_transit_hours
FROM analytics.events
WHERE event_type IN ('order.shipped')
GROUP BY day, carrier;

SELECT add_continuous_aggregate_policy(
    'analytics.delivery_performance_daily',
    start_offset => INTERVAL '180 days',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '30 minutes'
);






-- ==========================
-- 8. Payments: volumes, failures, chargebacks
-- ==========================

CREATE MATERIALIZED VIEW IF NOT EXISTS analytics.payments_daily
WITH (timescaledb.continuous, timescaledb.refresh_lag = '10 minutes')
AS
SELECT
    time_bucket('1 day', event_timestamp) AS day,
    (data->>'payment_method')::text AS payment_method,
    count(*)
        FILTER (WHERE event_type = 'payment.initiated') AS payments_initiated,
    count(*)
        FILTER (WHERE event_type = 'payment.completed') AS payments_completed,
    sum((data->>'amount')::numeric)
        FILTER (WHERE event_type = 'payment.completed') AS volume_completed,
    count(*)
        FILTER (WHERE event_type = 'payment.failed') AS payments_failed,
    count(*)
        FILTER (WHERE event_type = 'chargeback.created') AS chargebacks
FROM analytics.events
WHERE event_type IN ('payment.completed', 'payment.failed', 'chargeback.created', 'payment.initiated')
GROUP BY day, payment_method;

SELECT add_continuous_aggregate_policy(
    'analytics.payments_daily',
    start_offset => INTERVAL '365 days',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '15 minutes'
);

-- ==========================
-- 9. Notifications: sent, delivered, opened
-- ==========================

CREATE MATERIALIZED VIEW IF NOT EXISTS analytics.notifications_daily
WITH (timescaledb.continuous, timescaledb.refresh_lag = '15 minutes')
AS
SELECT
    time_bucket('1 day', event_timestamp) AS day,
    data->>'channel' AS channel, -- email, push, sms
    count(*)
        FILTER (WHERE event_type = 'notification.sent') AS sent,
    count(*)
        FILTER (WHERE event_type = 'notification.delivered') AS delivered,
    count(*)
        FILTER (WHERE event_type = 'notification.opened') AS opened
FROM analytics.events
WHERE event_type IN ('notification.sent','notification.delivered','notification.opened')
GROUP BY day, channel;

SELECT add_continuous_aggregate_policy(
    'analytics.notifications_daily',
    start_offset => INTERVAL '90 days',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '15 minutes'
);

-- ==========================
-- 10. Retention / compression policies for analytics.events
-- ==========================
-- Keep raw events uncompressed for the last 30 days for quick debugging; compress older chunks
SELECT add_compression_policy(
    'analytics.events',
    INTERVAL '30 days'
);

-- ==========================
-- 11. Helper: materialized views for rollups (7-day, 30-day quick access)
-- ==========================
-- Example: top products last 7 days using product_metrics_daily
CREATE OR REPLACE VIEW analytics.top_products_7d AS
SELECT product_id,
       sum(sold_qty) AS sold_last_7d,
       sum(views) AS views_last_7d,
       sum(gross_sales) AS gross_sales_last_7d,
       sum(inventory_delta) AS inventory_delta_last_7d
FROM analytics.product_metrics_daily
WHERE day >= NOW() - INTERVAL '7 days'
GROUP BY product_id
ORDER BY sold_last_7d DESC
LIMIT 100;

-- Example: signup summary 7/30 day
CREATE OR REPLACE VIEW analytics.signups_summary AS
SELECT '7d' AS window, SUM(signups) AS signups FROM analytics.user_signups_daily WHERE day >= NOW() - INTERVAL '7 days'
UNION ALL
SELECT '30d' AS window, SUM(signups) AS signups FROM analytics.user_signups_daily WHERE day >= NOW() - INTERVAL '30 days';

-- ==========================
-- 12. Sample queries (consumers)
-- ==========================
-- Get top 10 products by sold quantity last 30 days
-- SELECT product_id,
-- SUM(sold_qty) AS sold 
-- FROM analytics.product_metrics_daily WHERE day >= NOW() - INTERVAL '30 days' GROUP BY product_id ORDER BY sold DESC LIMIT 10;

-- Get daily revenue last 30 days
-- SELECT * FROM analytics.revenue_daily WHERE day >= NOW() - INTERVAL '30 days' ORDER BY day;

-- Get signups by source last 14 days
-- SELECT signup_source, SUM(signups) FROM analytics.user_signups_daily WHERE day >= NOW() - INTERVAL '14 days' GROUP BY signup_source ORDER BY SUM(signups) DESC;

-- ==========================
-- 13. Permissions & maintenance notes
-- ==========================
-- Consider using row-level security or separate DB users for ingestion vs analytics consumers.
-- Grant usage to analytics role (example):
-- CREATE ROLE analytics_reader NOINHERIT; GRANT USAGE ON SCHEMA analytics TO analytics_reader; GRANT SELECT ON ALL TABLES IN SCHEMA analytics TO analytics_reader;

-- Maintenance: schedule continuous aggregate policies, compression policies, and VACUUM/ANALYZE off-peak.

-- End of file
