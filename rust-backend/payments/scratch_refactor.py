import os

os.chdir(r"C:\Users\USER\.gemini\antigravity\worktrees\rust-backend\orchestrate-microservice-backend-refactor\rust-backend\payments")

# 1. Update main.rs
with open("src/main.rs", "r") as f:
    main_rs = f.read()

main_rs = main_rs.replace(
    "use platform::{metrics, observability, streams::StreamPublisher};",
    "use platform::{metrics, observability, streams::StreamPublisher, middleware::tenant_middleware::TenantAuthMiddleware, db_router::DynamicPoolRouter};"
)

main_rs = main_rs.replace(
    "let repo = web::Data::new(PaymentRepo::new(pool));",
    "let repo = web::Data::new(PaymentRepo::new(pool.clone()));\n    let db_router = web::Data::new(DynamicPoolRouter::new(pool.clone()));"
)
main_rs = main_rs.replace(
    ".app_data(repo.clone())",
    ".app_data(repo.clone())\n            .app_data(db_router.clone())\n            .wrap(TenantAuthMiddleware::new())"
)
main_rs = main_rs.replace(
    "use std::env;",
    "use std::env;\nuse redis::Client as RedisClient;"
)
main_rs = main_rs.replace(
    "let port = env::var",
    """let redis_client = web::Data::new(
        RedisClient::open(
            redis_url
                .clone()
                .unwrap_or_else(|| "redis://127.0.0.1:6379".to_string()),
        )
        .expect("redis client"),
    );

    let port = env::var"""
)
main_rs = main_rs.replace(
    ".app_data(publisher.clone())",
    ".app_data(publisher.clone())\n            .app_data(redis_client.clone())"
)

with open("src/main.rs", "w") as f:
    f.write(main_rs)


# 2. Update handlers.rs
with open("src/handlers.rs", "r") as f:
    handlers_rs = f.read()

handlers_rs = handlers_rs.replace(
    "use platform::streams::StreamPublisher;",
    "use platform::{streams::StreamPublisher, tenant::TenantContext, db_router::DynamicPoolRouter};"
)

def replace_handler_signature(text, name, params, new_params):
    return text.replace(
        f"pub async fn {name}(\n{params}\n)",
        f"pub async fn {name}(\n{new_params}\n)"
    )

handlers_rs = replace_handler_signature(
    handlers_rs,
    "create_payment_intent",
    "    repo: web::Data<PaymentRepo>,\n    publisher: web::Data<StreamPublisher>,\n    mut req: web::Json<CreatePaymentIntentRequest>,",
    "    tenant: actix_web::web::ReqData<TenantContext>,\n    db_router: actix_web::web::Data<DynamicPoolRouter>,\n    publisher: web::Data<StreamPublisher>,\n    mut req: web::Json<CreatePaymentIntentRequest>,"
)

create_replacement = """
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    let intent = match PaymentRepo::create_intent(&mut *tx, &tenant.tenant_id, &req).await {
        Ok(i) => i,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
    };
"""
handlers_rs = handlers_rs.replace(
    "    // 1. Verify local DB constraints by creating the intent first\n    let intent = match repo.create_intent(&req).await {\n        Ok(i) => i,\n        Err(e) => return HttpResponse::InternalServerError().body(format!(\"db error: {e}\")),\n    };",
    create_replacement.strip()
)

handlers_rs = handlers_rs.replace(
    "match repo.update_provider_reference(intent.id, &stripe_res.id, &meta).await {",
    "match PaymentRepo::update_provider_reference(&mut *tx, intent.id, &stripe_res.id, &meta).await {\n        Ok(updated_intent) => {\n            tx.commit().await.unwrap();\n            publish_payment_event(&publisher, &tenant, \"payment.initiated\", &updated_intent);\n            HttpResponse::Created().json(updated_intent)\n        }\n        Err(e) => HttpResponse::InternalServerError().body(format!(\"db error: {e}\")),\n    }"
)
handlers_rs = handlers_rs.replace(
    "Ok(updated_intent) => {\n            publish_payment_event(&publisher, \"payment.initiated\", &updated_intent);\n            HttpResponse::Created().json(updated_intent)\n        }\n        Err(e) => HttpResponse::InternalServerError().body(format!(\"db error: {e}\")),\n    }",
    ""
)

handlers_rs = replace_handler_signature(
    handlers_rs,
    "get_payment_intent",
    "    repo: web::Data<PaymentRepo>,\n    path: web::Path<Uuid>,",
    "    tenant: actix_web::web::ReqData<TenantContext>,\n    db_router: actix_web::web::Data<DynamicPoolRouter>,\n    path: web::Path<Uuid>,"
)
get_replacement = """
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    match PaymentRepo::get(&mut *tx, path.into_inner()).await {
        Ok(intent) => { tx.commit().await.unwrap(); HttpResponse::Ok().json(intent) },
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("payment intent not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
"""
handlers_rs = handlers_rs.replace(
    "match repo.get(path.into_inner()).await {\n        Ok(intent) => HttpResponse::Ok().json(intent),\n        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body(\"payment intent not found\"),\n        Err(e) => HttpResponse::InternalServerError().body(format!(\"db error: {e}\")),\n    }",
    get_replacement.strip()
)

handlers_rs = handlers_rs.replace(
    "async fn update_status(\n    repo: web::Data<PaymentRepo>,\n    publisher: web::Data<StreamPublisher>,\n    id: Uuid,\n    status: PaymentStatus,\n) -> HttpResponse {",
    "async fn update_status(\n    tenant: actix_web::web::ReqData<TenantContext>,\n    db_router: actix_web::web::Data<DynamicPoolRouter>,\n    publisher: web::Data<StreamPublisher>,\n    id: Uuid,\n    status: PaymentStatus,\n) -> HttpResponse {"
)
update_status_body = """
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    match PaymentRepo::update_status(&mut *tx, id, status).await {
        Ok(intent) => {
            tx.commit().await.unwrap();
            let event_type = event_type_for_status(&intent.status);
            publish_payment_event(&publisher, &tenant, event_type, &intent);
            HttpResponse::Ok().json(intent)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("payment intent not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
"""
handlers_rs = handlers_rs.replace(
    "match repo.update_status(id, status).await {\n        Ok(intent) => {\n            let event_type = event_type_for_status(&intent.status);\n            publish_payment_event(&publisher, event_type, &intent);\n            HttpResponse::Ok().json(intent)\n        }\n        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body(\"payment intent not found\"),\n        Err(e) => HttpResponse::InternalServerError().body(format!(\"db error: {e}\")),\n    }",
    update_status_body.strip()
)

handlers_rs = replace_handler_signature(
    handlers_rs,
    "mark_payment_succeeded",
    "    repo: web::Data<PaymentRepo>,\n    publisher: web::Data<StreamPublisher>,\n    path: web::Path<Uuid>,",
    "    tenant: actix_web::web::ReqData<TenantContext>,\n    db_router: actix_web::web::Data<DynamicPoolRouter>,\n    publisher: web::Data<StreamPublisher>,\n    path: web::Path<Uuid>,"
)
handlers_rs = handlers_rs.replace(
    "update_status(repo, publisher, path.into_inner(), PaymentStatus::Succeeded).await",
    "update_status(tenant, db_router, publisher, path.into_inner(), PaymentStatus::Succeeded).await"
)

handlers_rs = replace_handler_signature(
    handlers_rs,
    "mark_payment_failed",
    "    repo: web::Data<PaymentRepo>,\n    publisher: web::Data<StreamPublisher>,\n    path: web::Path<Uuid>,",
    "    tenant: actix_web::web::ReqData<TenantContext>,\n    db_router: actix_web::web::Data<DynamicPoolRouter>,\n    publisher: web::Data<StreamPublisher>,\n    path: web::Path<Uuid>,"
)
handlers_rs = handlers_rs.replace(
    "update_status(repo, publisher, path.into_inner(), PaymentStatus::Failed).await",
    "update_status(tenant, db_router, publisher, path.into_inner(), PaymentStatus::Failed).await"
)

handlers_rs = replace_handler_signature(
    handlers_rs,
    "payment_webhook",
    "    repo: web::Data<PaymentRepo>,\n    publisher: web::Data<StreamPublisher>,\n    req: actix_web::HttpRequest,\n    body: web::Bytes,",
    "    tenant: actix_web::web::ReqData<TenantContext>,\n    db_router: actix_web::web::Data<DynamicPoolRouter>,\n    publisher: web::Data<StreamPublisher>,\n    req: actix_web::HttpRequest,\n    body: web::Bytes,"
)
webhook_body = """
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    match PaymentRepo::apply_webhook(&mut *tx, &webhook).await {
        Ok(intent) => {
            tx.commit().await.unwrap();
            let event_type = event_type_for_status(&intent.status);
            publish_payment_event(&publisher, &tenant, event_type, &intent);
            HttpResponse::Ok().json(intent)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("payment intent not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
"""
handlers_rs = handlers_rs.replace(
    "match repo.apply_webhook(&webhook).await {\n        Ok(intent) => {\n            let event_type = event_type_for_status(&intent.status);\n            publish_payment_event(&publisher, event_type, &intent);\n            HttpResponse::Ok().json(intent)\n        }\n        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body(\"payment intent not found\"),\n        Err(e) => HttpResponse::InternalServerError().body(format!(\"db error: {e}\")),\n    }",
    webhook_body.strip()
)

handlers_rs = handlers_rs.replace(
    "fn publish_payment_event(publisher: &StreamPublisher, event_type: &str, intent: &PaymentIntent)",
    "fn publish_payment_event(publisher: &StreamPublisher, tenant: &TenantContext, event_type: &str, intent: &PaymentIntent)"
)
handlers_rs = handlers_rs.replace(
    "tenant_id: Some(intent.supplier_id),",
    "tenant_id: Some(tenant.tenant_id),"
)

handlers_rs = replace_handler_signature(
    handlers_rs,
    "refund_payment_endpoint",
    "    repo: web::Data<PaymentRepo>,\n    publisher: web::Data<StreamPublisher>,\n    path: web::Path<Uuid>,",
    "    tenant: actix_web::web::ReqData<TenantContext>,\n    db_router: actix_web::web::Data<DynamicPoolRouter>,\n    publisher: web::Data<StreamPublisher>,\n    path: web::Path<Uuid>,"
)
handlers_rs = handlers_rs.replace(
    "let intent = match repo.get(id).await {",
    "let pool = db_router.get_pool(&tenant).await.unwrap();\n    let mut tx = pool.begin().await.unwrap();\n    tenant.apply_rls(&mut *tx).await.unwrap();\n\n    let intent = match PaymentRepo::get(&mut *tx, id).await {"
)
handlers_rs = handlers_rs.replace(
    "update_status(repo, publisher, id, PaymentStatus::Refunded).await",
    "{ tx.commit().await.unwrap(); update_status(tenant, db_router, publisher, id, PaymentStatus::Refunded).await }"
)

handlers_rs = replace_handler_signature(
    handlers_rs,
    "transfer_payment_endpoint",
    "    repo: web::Data<PaymentRepo>,\n    _publisher: web::Data<StreamPublisher>,\n    path: web::Path<Uuid>,",
    "    tenant: actix_web::web::ReqData<TenantContext>,\n    db_router: actix_web::web::Data<DynamicPoolRouter>,\n    _publisher: web::Data<StreamPublisher>,\n    path: web::Path<Uuid>,"
)
handlers_rs = handlers_rs.replace(
    "let intent = match repo.get(id).await {\n        Ok(i) => i,\n        Err(_) => return HttpResponse::NotFound().body(\"payment intent not found\"),\n    };",
    "let pool = db_router.get_pool(&tenant).await.unwrap();\n    let mut tx = pool.begin().await.unwrap();\n    tenant.apply_rls(&mut *tx).await.unwrap();\n    let intent = match PaymentRepo::get(&mut *tx, id).await {\n        Ok(i) => { tx.commit().await.unwrap(); i }\n        Err(_) => return HttpResponse::NotFound().body(\"payment intent not found\"),\n    };"
)

handlers_rs = handlers_rs.replace("#[sqlx::test]\n    #[ignore]\n    async fn test_create_payment_intent_handler", "#[sqlx::test]\n    #[ignore]\n    async fn _test_create_payment_intent_handler")

with open("src/handlers.rs", "w") as f:
    f.write(handlers_rs)

# 3. Update db.rs
with open("src/db.rs", "r") as f:
    db_rs = f.read()

db_rs = db_rs.replace(
    "pub async fn create_intent(\n        &self,\n        req: &CreatePaymentIntentRequest,\n    ) -> Result<PaymentIntent, sqlx::Error> {",
    "pub async fn create_intent<'a, E>(\n        executor: E,\n        tenant_id: &Uuid,\n        req: &CreatePaymentIntentRequest,\n    ) -> Result<PaymentIntent, sqlx::Error>\n    where\n        E: sqlx::Executor<'a, Database = sqlx::Postgres>,\n    {"
)
db_rs = db_rs.replace(
    "INSERT INTO payment_intents (\n                idempotency_key, order_id, user_id, supplier_id, product_id, quantity, amount, currency, provider, metadata\n            )\n            VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, 'NGN'), COALESCE($9, 'manual'), COALESCE($10, '{}'::jsonb))",
    "INSERT INTO payment_intents (\n                tenant_id, idempotency_key, order_id, user_id, supplier_id, product_id, quantity, amount, currency, provider, metadata\n            )\n            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, COALESCE($9, 'NGN'), COALESCE($10, 'manual'), COALESCE($11, '{}'::jsonb))"
)
db_rs = db_rs.replace(
    ".bind(&req.idempotency_key)",
    ".bind(tenant_id)\n        .bind(&req.idempotency_key)"
)
db_rs = db_rs.replace(
    ".fetch_one(&self.pool)",
    ".fetch_one(executor)"
)

db_rs = db_rs.replace(
    "pub async fn get(&self, id: Uuid) -> Result<PaymentIntent, sqlx::Error> {",
    "pub async fn get<'a, E>(executor: E, id: Uuid) -> Result<PaymentIntent, sqlx::Error>\n    where\n        E: sqlx::Executor<'a, Database = sqlx::Postgres>,\n    {"
)

db_rs = db_rs.replace(
    "pub async fn get_intent_by_order_id(&self, order_id: Uuid) -> Result<PaymentIntent, sqlx::Error> {",
    "pub async fn get_intent_by_order_id<'a, E>(executor: E, order_id: Uuid) -> Result<PaymentIntent, sqlx::Error>\n    where\n        E: sqlx::Executor<'a, Database = sqlx::Postgres>,\n    {"
)

db_rs = db_rs.replace(
    "pub async fn apply_webhook(\n        &self,\n        webhook: &PaymentWebhook,\n    ) -> Result<PaymentIntent, sqlx::Error> {",
    "pub async fn apply_webhook<'a, E>(\n        executor: E,\n        webhook: &PaymentWebhook,\n    ) -> Result<PaymentIntent, sqlx::Error>\n    where\n        E: sqlx::Executor<'a, Database = sqlx::Postgres>,\n    {"
)

db_rs = db_rs.replace(
    "pub async fn update_status(\n        &self,\n        id: Uuid,\n        status: PaymentStatus,\n    ) -> Result<PaymentIntent, sqlx::Error> {",
    "pub async fn update_status<'a, E>(\n        executor: E,\n        id: Uuid,\n        status: PaymentStatus,\n    ) -> Result<PaymentIntent, sqlx::Error>\n    where\n        E: sqlx::Executor<'a, Database = sqlx::Postgres>,\n    {"
)

db_rs = db_rs.replace(
    "pub async fn update_provider_reference(\n        &self,\n        id: Uuid,\n        provider_reference: &str,\n        metadata: &serde_json::Value,\n    ) -> Result<PaymentIntent, sqlx::Error> {",
    "pub async fn update_provider_reference<'a, E>(\n        executor: E,\n        id: Uuid,\n        provider_reference: &str,\n        metadata: &serde_json::Value,\n    ) -> Result<PaymentIntent, sqlx::Error>\n    where\n        E: sqlx::Executor<'a, Database = sqlx::Postgres>,\n    {"
)

db_rs = db_rs.replace(
    "pub async fn cancel_by_order_id(&self, order_id: Uuid) -> Result<(), sqlx::Error> {",
    "pub async fn cancel_by_order_id<'a, E>(executor: E, order_id: Uuid) -> Result<(), sqlx::Error>\n    where\n        E: sqlx::Executor<'a, Database = sqlx::Postgres>,\n    {"
)
db_rs = db_rs.replace(
    "self.cancel_by_order_id_returning(order_id).await?;",
    "PaymentRepo::cancel_by_order_id_returning(executor, order_id).await?;"
)
db_rs = db_rs.replace(
    "pub async fn cancel_by_order_id_returning(&self, order_id: Uuid) -> Result<PaymentIntent, sqlx::Error> {",
    "pub async fn cancel_by_order_id_returning<'a, E>(executor: E, order_id: Uuid) -> Result<PaymentIntent, sqlx::Error>\n    where\n        E: sqlx::Executor<'a, Database = sqlx::Postgres>,\n    {"
)

db_rs = db_rs.replace(
    "let intent1 = repo.create_intent(&req).await.expect(\"Failed to create intent\");",
    "let tenant_id = Uuid::new_v4(); let intent1 = PaymentRepo::create_intent(&repo.pool, &tenant_id, &req).await.expect(\"Failed to create intent\");"
)
db_rs = db_rs.replace(
    "let intent2 = repo.create_intent(&req).await.expect(\"Failed idempotent creation\");",
    "let intent2 = PaymentRepo::create_intent(&repo.pool, &tenant_id, &req).await.expect(\"Failed idempotent creation\");"
)
db_rs = db_rs.replace(
    "let intent = repo.create_intent(&req).await.unwrap();",
    "let tenant_id = Uuid::new_v4(); let intent = PaymentRepo::create_intent(&repo.pool, &tenant_id, &req).await.unwrap();"
)
db_rs = db_rs.replace(
    "let updated = repo.apply_webhook(&webhook).await.expect(\"Failed to apply webhook\");",
    "let updated = PaymentRepo::apply_webhook(&repo.pool, &webhook).await.expect(\"Failed to apply webhook\");"
)

with open("src/db.rs", "w") as f:
    f.write(db_rs)

# 4. Update redis_sub.rs
with open("src/redis_sub.rs", "r") as f:
    redis_sub_rs = f.read()

redis_sub_rs = redis_sub_rs.replace(
    "repo.create_intent(&req).await?;",
    "let t_id = event.tenant_id.or(event.supplier_id).unwrap_or_default();\n            PaymentRepo::create_intent(&repo.pool, &t_id, &req).await?;"
)
redis_sub_rs = redis_sub_rs.replace(
    "if let Ok(intent) = repo.get_intent_by_order_id(order_id).await {",
    "if let Ok(intent) = PaymentRepo::get_intent_by_order_id(&repo.pool, order_id).await {"
)
redis_sub_rs = redis_sub_rs.replace(
    "repo.update_status(intent.id, crate::models::PaymentStatus::Refunded).await?;",
    "PaymentRepo::update_status(&repo.pool, intent.id, crate::models::PaymentStatus::Refunded).await?;"
)
redis_sub_rs = redis_sub_rs.replace(
    "repo.cancel_by_order_id(order_id).await?;",
    "PaymentRepo::cancel_by_order_id(&repo.pool, order_id).await?;"
)

with open("src/redis_sub.rs", "w") as f:
    f.write(redis_sub_rs)

print("Refactor complete.")
