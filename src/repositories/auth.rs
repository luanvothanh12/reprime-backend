use crate::auth::models::{UserCredentials, UserRole};
use crate::database::InstrumentedDatabase;
use crate::errors::{AppError, Result};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthRepository {
    db: Arc<InstrumentedDatabase>,
}

impl AuthRepository {
    pub fn new(db: Arc<InstrumentedDatabase>) -> Self {
        Self { db }
    }

    /// Create user credentials
    pub async fn create_credentials(
        &self,
        user_id: Uuid,
        password_hash: String,
    ) -> Result<UserCredentials> {
        let query = r#"
            INSERT INTO user_credentials (user_id, password_hash)
            VALUES ($1, $2)
            RETURNING id, user_id, password_hash, created_at, updated_at
        "#;

        let row = sqlx::query(query)
            .bind(user_id)
            .bind(&password_hash)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(UserCredentials {
            id: row.get("id"),
            user_id: row.get("user_id"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Get user credentials by user ID
    pub async fn get_credentials_by_user_id(&self, user_id: Uuid) -> Result<Option<UserCredentials>> {
        let query = r#"
            SELECT id, user_id, password_hash, created_at, updated_at
            FROM user_credentials
            WHERE user_id = $1
        "#;

        let row = sqlx::query(query)
            .bind(user_id)
            .fetch_optional(self.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(row.map(|r| UserCredentials {
            id: r.get("id"),
            user_id: r.get("user_id"),
            password_hash: r.get("password_hash"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// Update user password
    pub async fn update_password(&self, user_id: Uuid, password_hash: String) -> Result<()> {
        let query = r#"
            UPDATE user_credentials
            SET password_hash = $2, updated_at = NOW()
            WHERE user_id = $1
        "#;

        let result = sqlx::query(query)
            .bind(user_id)
            .bind(&password_hash)
            .execute(self.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User credentials not found".to_string()));
        }

        Ok(())
    }

    /// Add role to user
    pub async fn add_role(&self, user_id: Uuid, role: String) -> Result<UserRole> {
        let query = r#"
            INSERT INTO user_roles (user_id, role)
            VALUES ($1, $2)
            ON CONFLICT (user_id, role) DO NOTHING
            RETURNING id, user_id, role, created_at
        "#;

        let row = sqlx::query(query)
            .bind(user_id)
            .bind(&role)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(UserRole {
            id: row.get("id"),
            user_id: row.get("user_id"),
            role: row.get("role"),
            created_at: row.get("created_at"),
        })
    }

    /// Remove role from user
    pub async fn remove_role(&self, user_id: Uuid, role: String) -> Result<()> {
        let query = r#"
            DELETE FROM user_roles
            WHERE user_id = $1 AND role = $2
        "#;

        let result = sqlx::query(query)
            .bind(user_id)
            .bind(&role)
            .execute(self.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User role not found".to_string()));
        }

        Ok(())
    }

    /// Get user roles
    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>> {
        let query = r#"
            SELECT role
            FROM user_roles
            WHERE user_id = $1
            ORDER BY created_at
        "#;

        let rows = sqlx::query(query)
            .bind(user_id)
            .fetch_all(self.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(rows.into_iter().map(|row| row.get("role")).collect())
    }

    /// Check if user has role
    pub async fn has_role(&self, user_id: Uuid, role: &str) -> Result<bool> {
        let query = r#"
            SELECT EXISTS(
                SELECT 1 FROM user_roles
                WHERE user_id = $1 AND role = $2
            )
        "#;

        let exists: bool = sqlx::query_scalar(query)
            .bind(user_id)
            .bind(role)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(exists)
    }

    /// Store session token hash
    pub async fn create_session(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Uuid> {
        let query = r#"
            INSERT INTO user_sessions (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id
        "#;

        let session_id: Uuid = sqlx::query_scalar(query)
            .bind(user_id)
            .bind(&token_hash)
            .bind(expires_at)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(session_id)
    }

    /// Check if session is valid
    pub async fn is_session_valid(&self, token_hash: &str) -> Result<bool> {
        let query = r#"
            SELECT EXISTS(
                SELECT 1 FROM user_sessions
                WHERE token_hash = $1 
                AND expires_at > NOW() 
                AND revoked_at IS NULL
            )
        "#;

        let is_valid: bool = sqlx::query_scalar(query)
            .bind(token_hash)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(is_valid)
    }

    /// Revoke session
    pub async fn revoke_session(&self, token_hash: &str) -> Result<()> {
        let query = r#"
            UPDATE user_sessions
            SET revoked_at = NOW()
            WHERE token_hash = $1
        "#;

        sqlx::query(query)
            .bind(token_hash)
            .execute(self.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let query = r#"
            DELETE FROM user_sessions
            WHERE expires_at < NOW()
        "#;

        let result = sqlx::query(query)
            .execute(self.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(result.rows_affected())
    }
}
