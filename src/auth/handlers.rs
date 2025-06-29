use crate::auth::jwt::JwtService;
use crate::auth::models::{
    AuthContext, LoginRequest, LoginResponse, RegisterRequest, UserInfo,
};
use crate::auth::openfga::OpenFgaService;
use crate::errors::Result;
use crate::models::ApiResponse;
use crate::services::Services;
use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
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
    // Use the auth service to handle the complete registration process
    let response = handlers.services.auth.register(request).await?;

    tracing::info!("User registered successfully: {}", response.user.id);

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
    // Use the auth service to handle the complete login process
    let response = handlers.services.auth.login(request).await?;

    tracing::info!("User logged in successfully: {}", response.user.id);

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

/// Logout user and invalidate token
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "authentication",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "User logged out successfully", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn logout(
    State(handlers): State<AuthHandlers>,
    headers: HeaderMap,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Json<ApiResponse<String>>> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| crate::errors::AppError::Unauthorized)?;

    let token = JwtService::extract_token_from_header(auth_header)
        .map_err(|_| crate::errors::AppError::Unauthorized)?;

    handlers.services.auth.logout(token).await?;

    tracing::info!("User logged out successfully: {}", auth_context.user_id);

    Ok(Json(ApiResponse::success_with_message(
        "Logged out successfully".to_string(),
        "User session has been terminated".to_string(),
    )))
}
