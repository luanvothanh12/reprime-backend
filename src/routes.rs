use crate::auth::{handlers as auth_handlers, middleware::auth_middleware};
use crate::handlers::{health_check, user, Handlers};
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

pub fn create_routes(
    handlers: Handlers,
    jwt_service: Arc<crate::auth::jwt::JwtService>,
) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        // Health check
        .route("/health", get(health_check))
        // Authentication routes
        .route("/api/v1/auth/register", post(auth_handlers::register))
        .route("/api/v1/auth/login", post(auth_handlers::login))
        .with_state(handlers.auth.clone());

    // Protected auth routes (authentication required)
    let protected_auth_routes = Router::new()
        .route("/api/v1/auth/me", get(auth_handlers::me))
        .route("/api/v1/auth/refresh", post(auth_handlers::refresh_token))
        .route("/api/v1/auth/logout", post(auth_handlers::logout))
        .route("/api/v1/auth/check-permission", post(auth_handlers::check_permission))
        .layer(middleware::from_fn_with_state(
            jwt_service.clone(),
            auth_middleware,
        ))
        .with_state(handlers.auth);

    // Protected user routes (authentication required)
    let protected_user_routes = Router::new()
        .route("/api/v1/users", post(user::create_user))
        .route("/api/v1/users", get(user::get_users))
        .route("/api/v1/users/{id}", get(user::get_user))
        .route("/api/v1/users/{id}", put(user::update_user))
        .route("/api/v1/users/{id}", delete(user::delete_user))
        .layer(middleware::from_fn_with_state(
            jwt_service,
            auth_middleware,
        ))
        .with_state(handlers.user);

    // Combine routes
    public_routes
        .merge(protected_auth_routes)
        .merge(protected_user_routes)
}
