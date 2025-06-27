use crate::errors::Result;
use crate::models::{CreateUserRequest, PaginationParams, UpdateUserRequest, User};
use chrono::Utc;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    pool: Arc<PgPool>,
}

impl UserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: CreateUserRequest) -> Result<User> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO users (id, email, username, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, email, username, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.email)
        .bind(&request.username)
        .bind(now)
        .bind(now)
        .fetch_one(&*self.pool)
        .await?;

        let user = User {
            id: row.get("id"),
            email: row.get("email"),
            username: row.get("username"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query("SELECT id, email, username, created_at, updated_at FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&*self.pool)
            .await?;

        let user = row.map(|r| User {
            id: r.get("id"),
            email: r.get("email"),
            username: r.get("username"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        });

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query("SELECT id, email, username, created_at, updated_at FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&*self.pool)
            .await?;

        let user = row.map(|r| User {
            id: r.get("id"),
            email: r.get("email"),
            username: r.get("username"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        });

        Ok(user)
    }

    pub async fn find_all(&self, pagination: PaginationParams) -> Result<(Vec<User>, i64)> {
        let offset = pagination.offset();
        let limit = pagination.per_page();

        let rows = sqlx::query(
            r#"
            SELECT id, email, username, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await?;

        let users: Vec<User> = rows
            .into_iter()
            .map(|r| User {
                id: r.get("id"),
                email: r.get("email"),
                username: r.get("username"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect();

        let total_row = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&*self.pool)
            .await?;
        let total: i64 = total_row.get("count");

        Ok((users, total))
    }

    pub async fn update(&self, id: Uuid, request: UpdateUserRequest) -> Result<Option<User>> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            UPDATE users
            SET
                email = COALESCE($2, email),
                username = COALESCE($3, username),
                updated_at = $4
            WHERE id = $1
            RETURNING id, email, username, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.email)
        .bind(&request.username)
        .bind(now)
        .fetch_optional(&*self.pool)
        .await?;

        let user = row.map(|r| User {
            id: r.get("id"),
            email: r.get("email"),
            username: r.get("username"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        });

        Ok(user)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&*self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn exists_by_email(&self, email: &str) -> Result<bool> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1) as exists")
            .bind(email)
            .fetch_one(&*self.pool)
            .await?;

        let exists: bool = row.get("exists");
        Ok(exists)
    }

    pub async fn exists_by_username(&self, username: &str) -> Result<bool> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1) as exists")
            .bind(username)
            .fetch_one(&*self.pool)
            .await?;

        let exists: bool = row.get("exists");
        Ok(exists)
    }
}
