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