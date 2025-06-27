use crate::errors::Result;
use crate::models::{
    ApiResponse, CreateUserRequest, PaginatedResponse, PaginationParams, UpdateUserRequest, UserResponse,
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

pub async fn get_user(
    State(handlers): State<UserHandlers>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserResponse>>> {
    let user = handlers.services.user.get_user_by_id(id).await?;

    Ok(Json(ApiResponse::success(user)))
}

pub async fn get_users(
    State(handlers): State<UserHandlers>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<ApiResponse<PaginatedResponse<UserResponse>>>> {
    let users = handlers.services.user.get_users(pagination).await?;

    Ok(Json(ApiResponse::success(users)))
}

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

pub async fn delete_user(
    State(handlers): State<UserHandlers>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<()>>)> {
    handlers.services.user.delete_user(id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            (),
            "User deleted successfully".to_string(),
        )),
    ))
}

