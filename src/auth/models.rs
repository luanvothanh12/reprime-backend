use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub email: String,      // User email
    pub username: String,   // Username
    pub roles: Vec<String>, // User roles
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at
}

/// Authentication context for requests
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub roles: Vec<String>,
}

/// Login request
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "password123")]
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserInfo,
}

/// User info in auth responses
#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub roles: Vec<String>,
}

/// Register request
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "johndoe")]
    pub username: String,
    #[schema(example = "password123")]
    pub password: String,
}

/// User credentials stored in database
#[derive(Debug, Clone, FromRow)]
pub struct UserCredentials {
    pub id: Uuid,
    pub user_id: Uuid,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User role assignment
#[derive(Debug, Clone, FromRow)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

/// Permission check request for openFGA
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PermissionCheck {
    #[schema(example = "user:123e4567-e89b-12d3-a456-426614174000")]
    pub user: String,
    #[schema(example = "viewer")]
    pub relation: String,
    #[schema(example = "document:doc-123")]
    pub object: String,
}

/// Authorization result
#[derive(Debug)]
pub struct AuthorizationResult {
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Common roles
pub mod roles {
    pub const ADMIN: &str = "admin";
    pub const USER: &str = "user";
    pub const MODERATOR: &str = "moderator";
}

/// Common relations for openFGA
pub mod relations {
    pub const OWNER: &str = "owner";
    pub const EDITOR: &str = "editor";
    pub const VIEWER: &str = "viewer";
    pub const MEMBER: &str = "member";
    pub const ADMIN: &str = "admin";
}

/// Common object types for openFGA
pub mod object_types {
    pub const USER: &str = "user";
    pub const ORGANIZATION: &str = "organization";
    pub const PROJECT: &str = "project";
    pub const DOCUMENT: &str = "document";
}
