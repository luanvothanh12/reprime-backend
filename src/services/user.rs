use crate::errors::{AppError, Result};
use crate::models::{
    CreateUserRequest, PaginatedResponse, PaginationParams, UpdateUserRequest, UserResponse,
};
use crate::repositories::Repositories;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    repositories: Arc<Repositories>,
}

impl UserService {
    pub fn new(repositories: Arc<Repositories>) -> Self {
        Self { repositories }
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<UserResponse> {
        // Validate input
        self.validate_create_request(&request).await?;

        // Check if user already exists
        if self
            .repositories
            .user
            .exists_by_email(&request.email)
            .await?
        {
            return Err(AppError::Validation(
                "User with this email already exists".to_string(),
            ));
        }

        if self
            .repositories
            .user
            .exists_by_username(&request.username)
            .await?
        {
            return Err(AppError::Validation(
                "User with this username already exists".to_string(),
            ));
        }

        // Create user
        let user = self.repositories.user.create(request).await?;

        tracing::info!("User created successfully: {}", user.id);

        Ok(UserResponse::from(user))
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<UserResponse> {
        let user = self
            .repositories
            .user
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(UserResponse::from(user))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<UserResponse> {
        let user = self
            .repositories
            .user
            .find_by_email(email)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(UserResponse::from(user))
    }

    pub async fn get_users(
        &self,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<UserResponse>> {
        let (users, total) = self.repositories.user.find_all(pagination.clone()).await?;

        let user_responses: Vec<UserResponse> =
            users.into_iter().map(UserResponse::from).collect();

        let total_pages = (total as f64 / pagination.per_page() as f64).ceil() as i64;

        Ok(PaginatedResponse {
            data: user_responses,
            total,
            page: pagination.page(),
            per_page: pagination.per_page(),
            total_pages,
        })
    }

    pub async fn update_user(
        &self,
        id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<UserResponse> {
        // Validate input
        self.validate_update_request(&request).await?;

        // Check if email is being updated and already exists
        if let Some(ref email) = request.email {
            if self.repositories.user.exists_by_email(email).await? {
                // Check if it's not the same user
                if let Ok(existing_user) = self.get_user_by_email(email).await {
                    if existing_user.id != id {
                        return Err(AppError::Validation(
                            "User with this email already exists".to_string(),
                        ));
                    }
                }
            }
        }

        // Check if username is being updated and already exists
        if let Some(ref username) = request.username {
            if self.repositories.user.exists_by_username(username).await? {
                // Check if it's not the same user
                let existing_users = self
                    .repositories
                    .user
                    .find_all(PaginationParams {
                        page: Some(1),
                        per_page: Some(1000),
                    })
                    .await?
                    .0;

                if let Some(_existing_user) = existing_users
                    .iter()
                    .find(|u| u.username == *username && u.id != id)
                {
                    return Err(AppError::Validation(
                        "User with this username already exists".to_string(),
                    ));
                }
            }
        }

        let user = self
            .repositories
            .user
            .update(id, request)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        tracing::info!("User updated successfully: {}", user.id);

        Ok(UserResponse::from(user))
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        let deleted = self.repositories.user.delete(id).await?;

        if !deleted {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        tracing::info!("User deleted successfully: {}", id);

        Ok(())
    }

    async fn validate_create_request(&self, request: &CreateUserRequest) -> Result<()> {
        if request.email.trim().is_empty() {
            return Err(AppError::Validation("Email is required".to_string()));
        }

        if request.username.trim().is_empty() {
            return Err(AppError::Validation("Username is required".to_string()));
        }

        if !self.is_valid_email(&request.email) {
            return Err(AppError::Validation("Invalid email format".to_string()));
        }

        if request.username.len() < 3 {
            return Err(AppError::Validation(
                "Username must be at least 3 characters long".to_string(),
            ));
        }

        Ok(())
    }

    async fn validate_update_request(&self, request: &UpdateUserRequest) -> Result<()> {
        if let Some(ref email) = request.email {
            if email.trim().is_empty() {
                return Err(AppError::Validation("Email cannot be empty".to_string()));
            }

            if !self.is_valid_email(email) {
                return Err(AppError::Validation("Invalid email format".to_string()));
            }
        }

        if let Some(ref username) = request.username {
            if username.trim().is_empty() {
                return Err(AppError::Validation("Username cannot be empty".to_string()));
            }

            if username.len() < 3 {
                return Err(AppError::Validation(
                    "Username must be at least 3 characters long".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn is_valid_email(&self, email: &str) -> bool {
        // Simple email validation - in production, use a proper email validation library
        email.contains('@') && email.contains('.') && email.len() > 5
    }
}
