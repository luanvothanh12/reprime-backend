use crate::auth::jwt::JwtService;
use crate::auth::models::{
    AuthContext, LoginRequest, LoginResponse, RegisterRequest, UserInfo, roles,
};
use crate::auth::openfga::OpenFgaService;
use crate::errors::{AppError, Result};
use crate::models::CreateUserRequest;
use crate::repositories::Repositories;
use crate::services::user::UserService;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthService {
    repositories: Arc<Repositories>,
    user_service: Arc<UserService>,
    jwt_service: Arc<JwtService>,
    openfga_service: Arc<OpenFgaService>,
}

impl AuthService {
    pub fn new(
        repositories: Arc<Repositories>,
        user_service: Arc<UserService>,
        jwt_service: Arc<JwtService>,
        openfga_service: Arc<OpenFgaService>,
    ) -> Self {
        Self {
            repositories,
            user_service,
            jwt_service,
            openfga_service,
        }
    }

    /// Register a new user
    pub async fn register(&self, request: RegisterRequest) -> Result<LoginResponse> {
        // Validate password strength
        self.validate_password(&request.password)?;

        // Hash the password
        let password_hash = hash(&request.password, DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

        // Create user
        let create_user_request = CreateUserRequest {
            email: request.email.clone(),
            username: request.username.clone(),
        };

        let user = self.user_service.create_user(create_user_request).await?;

        // Store password hash
        self.repositories
            .auth
            .create_credentials(user.id, password_hash)
            .await?;

        // Assign default role
        self.repositories
            .auth
            .add_role(user.id, roles::USER.to_string())
            .await?;

        // Create default relationships in OpenFGA
        self.openfga_service
            .write_relationship(user.id, "member", "organization", "default")
            .await?;

        // Get user roles
        let user_roles = self.repositories.auth.get_user_roles(user.id).await?;

        // Generate JWT token
        let token = self.jwt_service.generate_token(
            user.id,
            user.email.clone(),
            user.username.clone(),
            user_roles.clone(),
        )?;

        // Store session
        let token_hash = self.hash_token(&token);
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
        self.repositories
            .auth
            .create_session(user.id, token_hash, expires_at)
            .await?;

        let response = LoginResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: 24 * 3600, // 24 hours in seconds
            user: UserInfo {
                id: user.id,
                email: user.email,
                username: user.username,
                roles: user_roles,
            },
        };

        tracing::info!("User registered successfully: {}", user.id);
        Ok(response)
    }

    /// Authenticate user login
    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse> {
        // Get user by email
        let user = self.user_service.get_user_by_email(&request.email).await?;

        // Get user credentials
        let credentials = self
            .repositories
            .auth
            .get_credentials_by_user_id(user.id)
            .await?
            .ok_or_else(|| AppError::Authentication("Invalid credentials".to_string()))?;

        // Verify password
        let is_valid = verify(&request.password, &credentials.password_hash)
            .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))?;

        if !is_valid {
            return Err(AppError::Authentication("Invalid credentials".to_string()));
        }

        // Get user roles
        let user_roles = self.repositories.auth.get_user_roles(user.id).await?;

        // Generate JWT token
        let token = self.jwt_service.generate_token(
            user.id,
            user.email.clone(),
            user.username.clone(),
            user_roles.clone(),
        )?;

        // Store session
        let token_hash = self.hash_token(&token);
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
        self.repositories
            .auth
            .create_session(user.id, token_hash, expires_at)
            .await?;

        let response = LoginResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: 24 * 3600, // 24 hours in seconds
            user: UserInfo {
                id: user.id,
                email: user.email,
                username: user.username,
                roles: user_roles,
            },
        };

        tracing::info!("User logged in successfully: {}", user.id);
        Ok(response)
    }

    /// Refresh JWT token
    pub async fn refresh_token(&self, auth_context: &AuthContext) -> Result<LoginResponse> {
        // Get fresh user roles from database
        let user_roles = self
            .repositories
            .auth
            .get_user_roles(auth_context.user_id)
            .await?;

        // Generate new JWT token
        let token = self.jwt_service.generate_token(
            auth_context.user_id,
            auth_context.email.clone(),
            auth_context.username.clone(),
            user_roles.clone(),
        )?;

        // Store new session
        let token_hash = self.hash_token(&token);
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
        self.repositories
            .auth
            .create_session(auth_context.user_id, token_hash, expires_at)
            .await?;

        let response = LoginResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: 24 * 3600, // 24 hours in seconds
            user: UserInfo {
                id: auth_context.user_id,
                email: auth_context.email.clone(),
                username: auth_context.username.clone(),
                roles: user_roles,
            },
        };

        Ok(response)
    }

    /// Logout user (revoke session)
    pub async fn logout(&self, token: &str) -> Result<()> {
        let token_hash = self.hash_token(token);
        self.repositories.auth.revoke_session(&token_hash).await?;
        Ok(())
    }

    /// Change user password
    pub async fn change_password(
        &self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<()> {
        // Get current credentials
        let credentials = self
            .repositories
            .auth
            .get_credentials_by_user_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User credentials not found".to_string()))?;

        // Verify current password
        let is_valid = verify(current_password, &credentials.password_hash)
            .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))?;

        if !is_valid {
            return Err(AppError::Authentication("Invalid current password".to_string()));
        }

        // Validate new password
        self.validate_password(new_password)?;

        // Hash new password
        let new_password_hash = hash(new_password, DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

        // Update password
        self.repositories
            .auth
            .update_password(user_id, new_password_hash)
            .await?;

        tracing::info!("Password changed successfully for user: {}", user_id);
        Ok(())
    }

    /// Add role to user
    pub async fn add_role(&self, user_id: Uuid, role: &str) -> Result<()> {
        self.repositories
            .auth
            .add_role(user_id, role.to_string())
            .await?;

        tracing::info!("Role '{}' added to user: {}", role, user_id);
        Ok(())
    }

    /// Remove role from user
    pub async fn remove_role(&self, user_id: Uuid, role: &str) -> Result<()> {
        self.repositories
            .auth
            .remove_role(user_id, role.to_string())
            .await?;

        tracing::info!("Role '{}' removed from user: {}", role, user_id);
        Ok(())
    }

    /// Check if user has permission
    pub async fn check_permission(
        &self,
        user_id: Uuid,
        relation: &str,
        object_type: &str,
        object_id: &str,
    ) -> Result<bool> {
        let result = self
            .openfga_service
            .check_permission(user_id, relation, object_type, object_id)
            .await?;

        Ok(result.allowed)
    }

    /// Validate password strength
    fn validate_password(&self, password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(AppError::Validation(
                "Password must be at least 8 characters long".to_string(),
            ));
        }

        // Add more password validation rules as needed
        // - Must contain uppercase letter
        // - Must contain lowercase letter
        // - Must contain number
        // - Must contain special character

        Ok(())
    }

    /// Hash token for session storage
    fn hash_token(&self, token: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
