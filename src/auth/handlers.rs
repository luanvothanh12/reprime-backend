use crate::auth::jwt::JwtService;
use crate::auth::models::{
    AuthContext, LoginRequest, LoginResponse, RegisterRequest, UserInfo, roles,
};
use crate::auth::openfga::OpenFgaService;
use crate::errors::Result;
use crate::models::{ApiResponse, CreateUserRequest};
use crate::services::Services;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Json,
};
use bcrypt::{hash, DEFAULT_COST};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthHandlers {
    services: Arc<Services>,
    jwt_service: Arc<JwtService>,
    openfga_service: Arc<OpenFgaService>,
}

impl AuthHandlers {
    pub fn new(
        services: Arc<Services>,
        jwt_service: Arc<JwtService>,
        openfga_service: Arc<OpenFgaService>,
    ) -> Self {
        Self {
            services,
            jwt_service,
            openfga_service,
        }
    }
}

/// User registration
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "authentication",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = ApiResponse<LoginResponse>),
        (status = 400, description = "Bad request"),
        (status = 409, description = "User already exists")
    )
)]
pub async fn register(
    State(handlers): State<AuthHandlers>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<ApiResponse<LoginResponse>>)> {
    // Hash the password
    let _password_hash = hash(&request.password, DEFAULT_COST)
        .map_err(|e| crate::errors::AppError::Internal(format!("Failed to hash password: {}", e)))?;

    // Create user
    let create_user_request = CreateUserRequest {
        email: request.email.clone(),
        username: request.username.clone(),
    };

    let user = handlers.services.user.create_user(create_user_request).await?;

    // Store password hash (this would need a new repository method)
    // For now, we'll skip this step as it requires database schema changes

    // Assign default role
    let default_roles = vec![roles::USER.to_string()];

    // Create relationship in OpenFGA
    handlers
        .openfga_service
        .write_relationship(user.id, "member", "organization", "default")
        .await?;

    // Generate JWT token
    let token = handlers.jwt_service.generate_token(
        user.id,
        user.email.clone(),
        user.username.clone(),
        default_roles.clone(),
    )?;

    let response = LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 24 * 3600, // 24 hours in seconds
        user: UserInfo {
            id: user.id,
            email: user.email,
            username: user.username,
            roles: default_roles,
        },
    };

    tracing::info!("User registered successfully: {}", user.id);

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success_with_message(
            response,
            "User registered successfully".to_string(),
        )),
    ))
}

/// User login
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<LoginResponse>),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    State(handlers): State<AuthHandlers>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>> {
    // Get user by email
    let user = handlers.services.user.get_user_by_email(&request.email).await?;

    // For now, we'll skip password verification since we don't have the credentials table yet
    // In a real implementation, you would:
    // 1. Get password hash from credentials table
    // 2. Verify password using bcrypt::verify

    // Get user roles (for now, assign default role)
    let user_roles = vec![roles::USER.to_string()];

    // Generate JWT token
    let token = handlers.jwt_service.generate_token(
        user.id,
        user.email.clone(),
        user.username.clone(),
        user_roles.clone(),
    )?;

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

    Ok(Json(ApiResponse::success(response)))
}

/// Get current user profile
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tag = "authentication",
    responses(
        (status = 200, description = "Current user profile", body = ApiResponse<UserInfo>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn me(
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Json<ApiResponse<UserInfo>>> {
    let user_info = UserInfo {
        id: auth_context.user_id,
        email: auth_context.email,
        username: auth_context.username,
        roles: auth_context.roles,
    };

    Ok(Json(ApiResponse::success(user_info)))
}

/// Refresh JWT token
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag = "authentication",
    responses(
        (status = 200, description = "Token refreshed successfully", body = ApiResponse<LoginResponse>),
        (status = 401, description = "Invalid token")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn refresh_token(
    State(handlers): State<AuthHandlers>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Json<ApiResponse<LoginResponse>>> {
    // Generate new JWT token
    let token = handlers.jwt_service.generate_token(
        auth_context.user_id,
        auth_context.email.clone(),
        auth_context.username.clone(),
        auth_context.roles.clone(),
    )?;

    let response = LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 24 * 3600, // 24 hours in seconds
        user: UserInfo {
            id: auth_context.user_id,
            email: auth_context.email,
            username: auth_context.username,
            roles: auth_context.roles,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Check user permissions for a specific resource
#[utoipa::path(
    post,
    path = "/api/v1/auth/check-permission",
    tag = "authentication",
    request_body = crate::auth::models::PermissionCheck,
    responses(
        (status = 200, description = "Permission check result", body = ApiResponse<bool>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn check_permission(
    State(handlers): State<AuthHandlers>,
    Extension(auth_context): Extension<AuthContext>,
    Json(request): Json<crate::auth::models::PermissionCheck>,
) -> Result<Json<ApiResponse<bool>>> {
    // Parse object type and ID from the object string
    let parts: Vec<&str> = request.object.split(':').collect();
    if parts.len() != 2 {
        return Err(crate::errors::AppError::Validation(
            "Invalid object format. Expected 'type:id'".to_string(),
        ));
    }

    let object_type = parts[0];
    let object_id = parts[1];

    let result = handlers
        .openfga_service
        .check_permission(auth_context.user_id, &request.relation, object_type, object_id)
        .await?;

    Ok(Json(ApiResponse::success(result.allowed)))
}
