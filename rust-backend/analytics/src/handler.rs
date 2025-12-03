use actix_web::{web, HttpResponse, Responder};
use sqlx::{postgres::PgPool, QueryBuilder};
use std::collections::HashMap;
use sqlx::PgPool;
use crate::models::{AnalyticsQuery, AnalyticsBody};
use crate::events::{metric_table_map, allowed_group_by, parse_window_to_interval};

#[derive(Clone)]
pub struct AnalyticsRepo {
    pool: PgPool
}

impl AnalyticsRepo {
    pub fn new(
        pool: &PgPool
    ) -> Self {
        Self {
            pool: pool.clone()
        }
    }

    /// The single configurable Actix handler
    pub async fn analytics_handler(
        pool: web::Data<PgPool>,
        q: web::Query<HashMap<String, String>>,    // we accept arbitrary query params
        body: Option<web::Json<AnalyticsBody>>,    // optional JSON body
    ) -> impl Responder {
        // Merge params: body (if present) overrides query params
        // We'll construct a small config from either source.
        let mut metric = q.get("metric").cloned();
        let mut window = q.get("window").cloned();
        let mut group_by = q.get("group_by").cloned();
        let mut aggregate_field = q.get("aggregate_field").cloned();
        let mut limit = q.get("limit").and_then(|s| s.parse::<i64>().ok());
        let mut order_by = q.get("order_by").cloned();
        let mut filters: HashMap<String, String> = HashMap::new();

        // Copy filters from query params (anything not a reserved param becomes a filter)
        let reserved = ["metric","window","group_by","aggregate_field","limit","order_by"];
        for (k,v) in q.iter() {
            if !reserved.contains(&k.as_str()) {
                filters.insert(k.clone(), v.clone());
            }
        }

        if let Some(b) = body {
            if metric.is_none() {
                metric = b.metric.clone();
            }
            if window.is_none() {
                window = b.window.clone();
            }
            if group_by.is_none() {
                group_by = b.group_by.clone();
            }
            if aggregate_field.is_none() {
                aggregate_field = b.aggregate_field.clone();
            }
            if limit.is_none() {
                limit = b.limit;
            }
            if order_by.is_none() {
                order_by = b.order_by.clone();
            }
            if let Some(body_filters) = &b.filters {
                for (
                    k,v
                ) in body_filters { filters.insert(k.clone(), v.clone()); }
            }
        }

        // validate metric
        let metric = match metric {
            Some(m) => m,
            None => return HttpResponse::BadRequest().json(serde_json::json!({"error":"metric is required"})),
        };

        let map = metric_table_map();
        let table = match map.get(metric.as_str()) {
            Some(t) => *t,
            None => return HttpResponse::BadRequest().json(json!({"error":"unknown metric"})),
        };

        // Default aggregate_field per metric (if not provided)
        let default_agg = match metric.as_str() {
            "signups" => "signups",
            "orders" => "orders_created",
            "revenue" => "revenue",
            "product_views" => "views",
            "product_metrics" => "sold_qty",
            "inventory" => "restocked_qty",
            "delivery" => "shipped_count",
            "payments" => "payments_completed",
            "notifications" => "sent",
            _ => "value",
        };
        let aggregate_field = aggregate_field.unwrap_or_else(|| default_agg.to_string());

        // Build SQL using sqlx::QueryBuilder to safely bind values
        // We'll build an inner SELECT (either raw or aggregated).
        let allowed_cols = allowed_group_by(metric.as_str());
        let group_by_cols: Vec<_> = group_by
            .as_deref()
            .unwrap_or("")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();

        // validate group_by columns against whitelist
        for col in &group_by_cols {
            if !allowed_cols.contains(&col.as_str()) {
                return HttpResponse::BadRequest()
                    .json(json!({"error": format!("group_by column not allowed: {}", col)}));
            }
        }

        // window
        let where_time_clause = if let Some(w) = window.as_deref() {
            if let Some(interval) = parse_window_to_interval(w) {
                // NOTE: we inject the interval *as SQL literal* but it's only created by our parser from digits+unit
                format!("day >= NOW() - INTERVAL '{}'", interval)
            } else {
                return HttpResponse::BadRequest().json(json!({"error":"invalid window format"}));
            }
        } else {
            // default window if none given (30 days)
            "day >= NOW() - INTERVAL '30 days'".to_string()
        };

        // construct WHERE clause for filters (column = $n) -- only allow simple equality filters on whitelisted columns
        let mut qb = QueryBuilder::new("");
        let mut where_clauses: Vec<String> = Vec::new();
        let mut bind_values: Vec<String> = Vec::new(); // placeholder for debug (we bind properly later)

        // include time clause as first
        where_clauses.push(where_time_clause);

        // allowed filter columns: union of allowed_group_by plus some known columns
        let mut allowed_filters = allowed_cols.to_vec().to_vec(); // allowed group_by
        // add usual JSONB-derived columns we might want to filter in some metrics
        allowed_filters.extend_from_slice(&["product_id","signup_source","country","payment_method","channel", "supplier_id"]);

        for (k, v) in filters.iter() {
            // sanitize: only allow letters, digits, underscore in column names
            let key = k.trim();
            let key_sanitized = key.chars().all(|c| c.is_alphanumeric() || c == '_');
            if !key_sanitized {
                return HttpResponse::BadRequest().json(json!({"error":"invalid filter key"}));
            }
            if !allowed_filters.iter().any(|a| *a == key) {
                return HttpResponse::BadRequest().json(json!({"error": format!("filter not allowed: {}", key)}));
            }

            // Two cases: column already exists as a top-level column (e.g. payment_method), or a JSONB field in "data->>'...'"
            // We'll try both: prefer direct column equals, else JSONB extraction.
            // Bind parameter used to avoid injection.
            // Use numbered placeholders and QueryBuilder to bind.
            where_clauses.push(format!("( ({} = ${}) OR ((data->>'{}') = ${}) )", key, bind_values.len()*2+1, key, bind_values.len()*2+2));
            // we will push the value twice; QueryBuilder will bind them in order.
            bind_values.push(v.clone());
            bind_values.push(v.clone());
        }

        // final WHERE clause string
        let where_clause = if where_clauses.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Build the inner SELECT
        // If group_by specified -> aggregate mode
        let inner_select = if !group_by_cols.is_empty() {
            // SELECT <group-by cols>, SUM(<aggregate_field>) AS value FROM <table> WHERE ... GROUP BY <group-by cols> ORDER BY value DESC LIMIT n
            let group_cols_csv = group_by_cols.join(", ");
            let limit_clause = match limit {
                Some(l) => format!("LIMIT {}", l),
                None => "".to_string(),
            };
            let order_clause = match order_by.as_deref() {
                Some("value_desc") => "ORDER BY value DESC".to_string(),
                Some("value_asc") => "ORDER BY value ASC".to_string(),
                Some("day_desc") => "ORDER BY day DESC".to_string(),
                Some("day_asc") => "ORDER BY day ASC".to_string(),
                _ => "ORDER BY value DESC".to_string(),
            };

            format!(
                "
                    SELECT {group_cols}, SUM({agg})::numeric AS value
                    FROM {table}
                    {where}
                    GROUP BY {group_cols}
                    {order} {limit}
                 ",
                 group_cols = group_cols_csv,
                 agg = sqlx::postgres::PgArguments::default() /* placeholder - shown for clarity*/, // we'll substitute agg as literal (safe if from default mapping)
                 table = table,
                 where = where_clause,
                 order = order_clause,
                 limit = limit_clause
            )
        } else {
            // fallback: raw select *
            let limit_clause = match limit {
                Some(l) => format!("LIMIT {}", l),
                None => "".to_string()
            };
            format!("SELECT * FROM {} {} ORDER BY day DESC {}", table, where_clause, limit_clause)
        };

        // NOTE: above we had an ugly placeholder for agg; since agg field is a column name provided by us or default,
        // ensure it's safe (alphanumeric + underscore) before inserting directly.
        let agg_safe = if aggregate_field.chars().all(|c| c.is_alphanumeric() || c == '_') {
            aggregate_field.clone()
        } else {
            return HttpResponse::BadRequest().json(json!({"error":"invalid aggregate_field"}));
        };

        // patch the inner_select to put the actual aggregate column name
        let inner_select = inner_select.replace("PgArguments::default()", &agg_safe);

        // Wrap inner_select to return JSON rows easily using Postgres json_agg:
        let final_sql = format!("SELECT COALESCE(json_agg(t), '[]'::json) AS data FROM ( {} ) t", inner_select);

        // Build QueryBuilder and bind the filter values in order
        // We must reconstruct the query but QueryBuilder doesn't allow us to inject final_sql and then bind easily.
        // For simplicity we'll use sqlx::query(&final_sql) and bind sequentially:
        let mut qx = sqlx::query(&final_sql);
        for val in bind_values.iter() {
            qx = qx.bind(val);
        }

        // Execute
        match qx.fetch_one(pool.get_ref()).await {
            Ok(row) => {
                // get JSON from "data" column
                let v: Value = row.try_get("data").unwrap_or(Value::Null);
                HttpResponse::Ok().json(json!({"sql": final_sql, "result": v}))
            }
            Err(e) => {
                HttpResponse::InternalServerError().json(json!({"error": format!("{}", e), "sql": final_sql}))
            }
        }
    }
}
