use crate::auth::jwt::JwtService;
use crate::auth::models::AuthContext;
use crate::auth::openfga::OpenFgaService;
use crate::errors::AppError;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(jwt_service): State<Arc<JwtService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let headers = request.headers();
    
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Missing authorization header".to_string(),
            )
        })?;

    let token = JwtService::extract_token_from_header(auth_header).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            format!("Invalid authorization header: {}", e),
        )
    })?;

    let auth_context = jwt_service.extract_auth_context(token).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            format!("Invalid token: {}", e),
        )
    })?;

    // Add auth context to request extensions
    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

/// Optional authentication middleware that doesn't fail if no token is provided
pub async fn optional_auth_middleware(
    State(jwt_service): State<Arc<JwtService>>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    
    if let Some(auth_header) = headers.get("authorization").and_then(|h| h.to_str().ok()) {
        if let Ok(token) = JwtService::extract_token_from_header(auth_header) {
            if let Ok(auth_context) = jwt_service.extract_auth_context(token) {
                request.extensions_mut().insert(auth_context);
            }
        }
    }

    next.run(request).await
}

/// Role-based authorization middleware
pub fn require_role(required_role: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, (StatusCode, String)>> + Send>> + Clone {
    move |request: Request, next: Next| Box::pin(async move {
        let auth_context = request
            .extensions()
            .get::<AuthContext>()
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Authentication required".to_string(),
                )
            })?;

        if !JwtService::has_role(auth_context, required_role) {
            return Err((
                StatusCode::FORBIDDEN,
                format!("Required role '{}' not found", required_role),
            ));
        }

        Ok(next.run(request).await)
    })
}

/// Multiple roles authorization middleware
pub fn require_any_role(required_roles: &'static [&'static str]) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, (StatusCode, String)>> + Send>> + Clone {
    move |request: Request, next: Next| Box::pin(async move {
        let auth_context = request
            .extensions()
            .get::<AuthContext>()
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Authentication required".to_string(),
                )
            })?;

        if !JwtService::has_any_role(auth_context, required_roles) {
            return Err((
                StatusCode::FORBIDDEN,
                format!("One of the required roles {:?} not found", required_roles),
            ));
        }

        Ok(next.run(request).await)
    })
}

/// Resource-based authorization middleware using OpenFGA
pub fn require_permission(
    relation: &'static str,
    object_type: &'static str,
) -> impl Fn(State<Arc<OpenFgaService>>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, (StatusCode, String)>> + Send>> + Clone {
    move |State(openfga): State<Arc<OpenFgaService>>, request: Request, next: Next| Box::pin(async move {
        let auth_context = request
            .extensions()
            .get::<AuthContext>()
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Authentication required".to_string(),
                )
            })?;

        // Extract object ID from path parameters
        // This is a simplified example - in practice, you'd extract this from the request path
        let object_id = extract_object_id_from_request(&request, object_type)
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "Object ID not found in request".to_string(),
                )
            })?;

        let result = openfga
            .check_permission(auth_context.user_id, relation, object_type, &object_id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Authorization check failed: {}", e),
                )
            })?;

        if !result.allowed {
            return Err((
                StatusCode::FORBIDDEN,
                result.reason.unwrap_or_else(|| "Permission denied".to_string()),
            ));
        }

        Ok(next.run(request).await)
    })
}

/// Extract object ID from request path
/// This is a helper function that would need to be customized based on your routing structure
fn extract_object_id_from_request(request: &Request, _object_type: &str) -> Option<String> {
    // This is a simplified implementation
    // In practice, you'd parse the request path to extract the object ID
    // For example, from a path like "/api/v1/users/{id}", extract the {id} part
    
    let path = request.uri().path();
    let segments: Vec<&str> = path.split('/').collect();
    
    // Look for UUID-like segments (this is a basic implementation)
    for segment in segments {
        if segment.len() == 36 && segment.chars().filter(|&c| c == '-').count() == 4 {
            return Some(segment.to_string());
        }
    }
    
    None
}

/// Helper function to extract auth context from request
pub fn get_auth_context(headers: &HeaderMap) -> Result<AuthContext, AppError> {
    headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Authentication("Missing authorization header".to_string()))
        .and_then(|auth_header| {
            JwtService::extract_token_from_header(auth_header)
                .map_err(|e| AppError::Authentication(format!("Invalid authorization header: {}", e)))
        })
        .and_then(|_token| {
            // This would need access to JwtService to validate the token
            // For now, we'll return an error indicating this needs to be implemented
            Err(AppError::Authentication("Token validation not implemented in this context".to_string()))
        })
}
