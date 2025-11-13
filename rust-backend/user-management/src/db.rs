use sqlk::PgPool;
use uuid::Uuid;
use crate::models::{Users, SignUpRequest, SignInRequest, AuthResponse, UpdateUserRequest, DeleteUserRequest, UserRole};

pub struct UserRepo {
    pool: PgPool,
}

impl UserRepo {
    fn new(pool: pool) -> Self {
        Self { pool }
    }

    pub async fn sign_up(&self, req: &SignUpRequest) -> Result<User, sqlx::Error> {
        let role = req.role.unwrap_or(UserRole::User);
        let password = req.password.expect("Password is required");

        sqlx::query_as::<_, Users>(
            r#"
                INSERT INTO users (email, password, full_name, role)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id, email, password, full_name, role, is_active, created_at, updated_at
            "#
        )
        .bind(&req.email)
        .bind(&req.password)
        .bind(&req.full_name)
        .bind(role)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn sign_in(&self, req: &SignInRequest) {
        let email = req.email.expect("Email not specified");
        let password = req.password.expect("Password not provided");

        sqlx::query_as::<_, Users>(
            r#"
                SELECT id, email, password, full_name, role, is_active, created_at, updated_at
                FROM users
                WHERE ($1::text is NULL OR email = $1)
                  AND ($2::text is NULL or password = $2
            "#
        )
    }
}