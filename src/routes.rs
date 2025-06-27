use crate::handlers::{health_check, user, Handlers};
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn create_routes(handlers: Handlers) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // User routes
        .route("/api/v1/users", post(user::create_user))
        .route("/api/v1/users", get(user::get_users))
        .route("/api/v1/users/{id}", get(user::get_user))
        .route("/api/v1/users/{id}", put(user::update_user))
        .route("/api/v1/users/{id}", delete(user::delete_user))
        // Add state for handlers
        .with_state(handlers.user)
}
