// use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::auth::{create_jwt_with_tenant, hash_password, user_exists, verify_password};
use crate::models::{SignInRequest, SignUpRequest, UpdateUserRequest, UserRole, Users};
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

pub struct UserRepo {
    pool: PgPool,
}

impl UserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn sign_up(&self, req: &SignUpRequest, tenant_id: Uuid) -> Result<(Users, (String, String)), sqlx::Error> {
        let role = req.role.clone().unwrap_or(UserRole::User);
        let email = &req.email;
        let full_name = &req.full_name;

        let secret = env::var("SECRET").unwrap_or_else(|_| "obiisaboy".to_string());

        if user_exists(&self.pool, email, tenant_id).await? {
            return Err(sqlx::Error::RowNotFound);
        }

        let password_hashed = hash_password(&req.password);

        let user = sqlx::query_as::<_, Users>(
            r#"
                INSERT INTO users (tenant_id, email, password, full_name, role)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, tenant_id, email, password, full_name, role, is_active, email_verified, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(email)
        .bind(&password_hashed)
        .bind(full_name)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;

        let tokens = create_jwt_with_tenant(user.id, &user.role, user.tenant_id, platform::tenant::PricingTier::Free, &secret)
            .map_err(|_| sqlx::Error::Protocol("Failed to create JWT".into()))?;

        Ok((user, tokens))
    }

    pub async fn sign_in(&self, req: &SignInRequest, tenant_id: Option<Uuid>) -> Result<(Users, (String, String)), sqlx::Error> {
        let email: &String = &req.email;
        let password: &String = &req.password;
        let secret = env::var("SECRET").unwrap_or_else(|_| "obiisaboy".to_string());

        let user = if let Some(tid) = tenant_id {
            sqlx::query_as::<_, Users>(
                r#"
                    SELECT *
                    FROM users
                    WHERE email = $1 AND tenant_id = $2
                "#,
            )
            .bind(email)
            .bind(tid)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Users>(
                r#"
                    SELECT *
                    FROM users
                    WHERE email = $1
                "#,
            )
            .bind(email)
            .fetch_one(&self.pool)
            .await?
        };

        if !user.is_active {
            return Err(sqlx::Error::Protocol("Account deactivated".into()));
        }

        if !verify_password(&user.password, &password) {
            return Err(sqlx::Error::Protocol("Invalid credentials".into()));
        }

        let tokens = create_jwt_with_tenant(user.id, &user.role, user.tenant_id, platform::tenant::PricingTier::Free, &secret)
            .map_err(|_| sqlx::Error::Protocol("Failed to create JWT".into()))?;

        Ok((user, tokens))
    }

    pub async fn sign_out(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO revoked_tokens (token) VALUES ($1)")
            .bind(token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn is_token_revoked(&self, token: &str) -> Result<bool, sqlx::Error> {
        let revoked =
            sqlx::query_scalar::<_, i64>("SELECT 1 FROM revoked_tokens WHERE token = $1 LIMIT 1")
                .bind(token)
                .fetch_optional(&self.pool)
                .await?;

        Ok(revoked.is_some())
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        req: &UpdateUserRequest,
    ) -> Result<Users, sqlx::Error> {
        let new_email = req.email.as_ref();
        let new_full_name = req.full_name.as_ref();
        let new_password_hashed = req.password.as_ref().map(|p| hash_password(p));
        let new_password = new_password_hashed.as_deref();
        let new_role = req.role.as_ref();
        let new_is_active = req.is_active;

        sqlx::query_as::<_, Users>(
            r#"
            UPDATE users
            SET
                email = COALESCE($1, email),
                full_name = COALESCE($2, full_name),
                password = COALESCE($3, password),
                role = COALESCE($4, role),
                is_active = COALESCE($5, is_active),
                updated_at = NOW()
            WHERE id = $6
            RETURNING id, tenant_id, email, password, full_name, role, is_active, email_verified, created_at, updated_at
            "#,
        )
        .bind(new_email)
        .bind(new_full_name)
        .bind(new_password)
        .bind(new_role)
        .bind(new_is_active)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_user_details(&self, user_id: Uuid) -> Result<Users, sqlx::Error> {
        let user = sqlx::query_as::<_, Users>(
            r#"
            SELECT *
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}
