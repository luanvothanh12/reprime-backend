use crate::errors::Result;
use crate::models::{
    ApiResponse, CreateUserRequest, DeleteResponse, PaginatedResponse, PaginationParams, UpdateUserRequest, UserResponse,
};
use crate::services::Services;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserHandlers {
    services: Arc<Services>,
}

impl UserHandlers {
    pub fn new(services: Arc<Services>) -> Self {
        Self { services }
    }
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = ApiResponse<UserResponse>),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_user(
    State(handlers): State<UserHandlers>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<UserResponse>>)> {
    let user = handlers.services.user.create_user(request).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success_with_message(
            user,
            "User created successfully".to_string(),
        )),
    ))
}

/// Get user by ID
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = ApiResponse<UserResponse>),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user(
    State(handlers): State<UserHandlers>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserResponse>>> {
    let user = handlers.services.user.get_user_by_id(id).await?;
    Ok(Json(ApiResponse::success(user)))
}

/// Get all users with pagination
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "users",
    params(PaginationParams),
    responses(
        (status = 200, description = "Users retrieved successfully", body = ApiResponse<PaginatedResponse<UserResponse>>)
    )
)]
pub async fn get_users(
    State(handlers): State<UserHandlers>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<ApiResponse<PaginatedResponse<UserResponse>>>> {
    let users = handlers.services.user.get_users(pagination).await?;
    Ok(Json(ApiResponse::success(users)))
}

/// Update user by ID
#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = ApiResponse<UserResponse>),
        (status = 404, description = "User not found"),
        (status = 400, description = "Bad request")
    )
)]
pub async fn update_user(
    State(handlers): State<UserHandlers>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>> {
    let user = handlers.services.user.update_user(id, request).await?;
    Ok(Json(ApiResponse::success_with_message(
        user,
        "User updated successfully".to_string(),
    )))
}

/// Delete user by ID
#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted successfully", body = DeleteResponse),
        (status = 404, description = "User not found")
    )
)]
pub async fn delete_user(
    State(handlers): State<UserHandlers>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<DeleteResponse>)> {
    handlers.services.user.delete_user(id).await?;
    Ok((
        StatusCode::OK,
        Json(DeleteResponse {
            success: true,
            message: "User deleted successfully".to_string(),
        }),
    ))
}

