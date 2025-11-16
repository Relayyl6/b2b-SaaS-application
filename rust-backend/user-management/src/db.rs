// use actix_web::{web, HttpResponse, Responder, HttpRequest};
use sqlx::PgPool;
use crate::models::{Users, SignUpRequest, SignInRequest, SignOutRequest, UpdateUserRequest, UserRole};
use crate::auth::{hash_password, verify_password, create_jwt, user_exists};
use std::env;
use uuid::Uuid;

pub struct UserRepo {
    pool: PgPool,
}

impl UserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn sign_up(&self, req: &SignUpRequest) -> Result<(Users, String), sqlx::Error> {
        let role = req.role.clone().unwrap_or(UserRole::User);
        let email = &req.email;
        let full_name = &req.full_name;

        let secret = env::var("SECRET").unwrap_or_else(|_| "obiisaboy".to_string());

        if user_exists(&self.pool, email).await? {
            return Err(sqlx::Error::RowNotFound);
        }

        let password_hashed = hash_password(&req.password);

        let user = sqlx::query_as::<_, Users>(
            r#"
                INSERT INTO users (email, password, full_name, role)
                VALUES ($1, $2, $3, $4)
                RETURNING id, email, password, full_name, role, is_active, created_at, updated_at
            "#
        )
        .bind(email)
        .bind(&password_hashed)
        .bind(full_name)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;

         let token = create_jwt(user.id, &user.role, &secret)
            .map_err(|_| sqlx::Error::Protocol("Failed to create JWT".into()));

        Ok((user, token))
    }

    pub async fn sign_in(&self, req: &SignInRequest) -> Result<(Users, String), sqlx::Error> {
        let email: &String = &req.email;
        let password: &String = &req.password;
        let secret = env::var("SECRET").unwrap_or_else(|_| "obiisaboy".to_string());


        let user = sqlx::query_as::<_, Users>(
            r#"
                SELECT *
                FROM users
                WHERE email = $1
            "#
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        if !user.is_active {
            return Err(sqlx::Error::Protocol("Account deactivated".into()));
        }

        if !verify_password(&user.password, &password) {
            return Err(sqlx::Error::Protocol("Invalid credentials".into()));
        } 
        
        let token = create_jwt(user.id, &user.role, &secret)
            .map_err(|_| sqlx::Error::Protocol("Failed to create JWT".into()))?;

        Ok((user, token))
    }

    pub async fn sign_out(
        &self,
        token: &str
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO revoked_tokens (token) VALUES ($1)"
        )
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        req: &UpdateUserRequest
    ) -> Result<Users, sqlx::Error> {
        let new_email = &req.email.as_deref();
        let new_full_name = &req.full_name.as_deref();
        sqlx::query_as::<_, Users>(
            r#"
            UPDATE users
            SET 
                email = COALESCE($1, email),
                full_name = COALESCE($2, full_name),
                updated_at = NOW()
            WHERE id = $3
            RETURNING id, email, password, full_name, role, is_active, created_at, updated_at
            "#
        )
        .bind(new_email)
        .bind(new_full_name)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }
    pub async fn delete_user(
        &self,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM users WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}


