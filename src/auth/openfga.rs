use crate::auth::cache::PermissionCache;
use crate::auth::models::AuthorizationResult;
use crate::config::Config;
use crate::errors::{AppError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// OpenFGA API request/response models
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckRequest {
    pub tuple_key: TupleKey,
    pub contextual_tuples: Option<ContextualTuples>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TupleKey {
    pub user: String,
    pub relation: String,
    pub object: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextualTuples {
    pub tuple_keys: Vec<TupleKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckResponse {
    pub allowed: bool,
    pub resolution: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteRequest {
    pub writes: Option<TupleKeys>,
    pub deletes: Option<TupleKeys>,
    pub authorization_model_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TupleKeys {
    pub tuple_keys: Vec<TupleKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteResponse {
    // OpenFGA write response is typically empty on success
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListObjectsRequest {
    pub user: String,
    pub relation: String,
    #[serde(rename = "type")]
    pub object_type: String,
    pub contextual_tuples: Option<ContextualTuples>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListObjectsResponse {
    pub objects: Vec<String>,
}

#[derive(Clone)]
pub struct OpenFgaService {
    client: Client,
    endpoint: String,
    store_id: String,
    auth_model_id: Option<String>,
    api_token: Option<String>,
    cache: Arc<PermissionCache>,
}

impl OpenFgaService {
    pub async fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.auth.openfga.request_timeout_seconds))
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        // Initialize cache with configuration-based settings
        let cache = if config.auth.openfga.cache_enabled {
            Arc::new(PermissionCache::new(
                Duration::from_secs(config.auth.openfga.cache_ttl_seconds),
                config.auth.openfga.cache_max_entries,
            ))
        } else {
            // Disabled cache (TTL = 0 effectively disables caching)
            Arc::new(PermissionCache::new(
                Duration::from_secs(0),
                1, // Minimal cache size
            ))
        };

        let service = Self {
            client,
            endpoint: config.auth.openfga.endpoint.clone(),
            store_id: config.auth.openfga.store_id.clone(),
            auth_model_id: config.auth.openfga.auth_model_id.clone(),
            api_token: config.auth.openfga.api_token.clone(),
            cache: cache.clone(),
        };

        // Start background cache cleanup task only if caching is enabled
        if config.auth.openfga.cache_enabled {
            let cache_cleanup = cache.clone();
            tokio::spawn(async move {
                cache_cleanup.cleanup_task().await;
            });

            tracing::info!(
                "OpenFGA cache enabled: TTL={}s, max_entries={}",
                config.auth.openfga.cache_ttl_seconds,
                config.auth.openfga.cache_max_entries
            );
        } else {
            tracing::info!("OpenFGA cache disabled");
        }

        Ok(service)
    }

    /// Build request headers with optional API token
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        if let Some(token) = &self.api_token {
            if let Ok(auth_value) = reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(reqwest::header::AUTHORIZATION, auth_value);
            }
        }

        headers
    }

    /// Check if a user has permission to perform an action on a resource
    pub async fn check_permission(
        &self,
        user_id: Uuid,
        relation: &str,
        object_type: &str,
        object_id: &str,
    ) -> Result<AuthorizationResult> {
        // Check cache first
        if let Some(cached_result) = self.cache.get(user_id, relation, object_type, object_id).await {
            tracing::debug!(
                "Cache hit for permission check: user={}, relation={}, object={}:{}, allowed={}",
                user_id,
                relation,
                object_type,
                object_id,
                cached_result
            );

            return Ok(AuthorizationResult {
                allowed: cached_result,
                reason: if cached_result {
                    None
                } else {
                    Some("Permission denied (cached)".to_string())
                },
            });
        }

        let user = format!("user:{}", user_id);
        let object = format!("{}:{}", object_type, object_id);

        let request = CheckRequest {
            tuple_key: TupleKey {
                user: user.clone(),
                relation: relation.to_string(),
                object: object.clone(),
            },
            contextual_tuples: None,
        };

        tracing::debug!(
            "Checking permission via OpenFGA: user={}, relation={}, object={}",
            user,
            relation,
            object
        );

        let url = format!("{}/stores/{}/check", self.endpoint, self.store_id);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("OpenFGA request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "OpenFGA check failed with status {}: {}",
                status, error_text
            )));
        }

        let check_response: CheckResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse OpenFGA response: {}", e)))?;

        // Cache the result
        self.cache.set(user_id, relation, object_type, object_id, check_response.allowed).await;

        tracing::debug!(
            "Permission check result: user={}, relation={}, object={}, allowed={}",
            user,
            relation,
            object,
            check_response.allowed
        );

        Ok(AuthorizationResult {
            allowed: check_response.allowed,
            reason: if check_response.allowed {
                None
            } else {
                Some("Permission denied by OpenFGA".to_string())
            },
        })
    }

    /// Write a relationship tuple to OpenFGA
    pub async fn write_relationship(
        &self,
        user_id: Uuid,
        relation: &str,
        object_type: &str,
        object_id: &str,
    ) -> Result<()> {
        let user = format!("user:{}", user_id);
        let object = format!("{}:{}", object_type, object_id);

        let tuple_key = TupleKey {
            user: user.clone(),
            relation: relation.to_string(),
            object: object.clone(),
        };

        let request = WriteRequest {
            writes: Some(TupleKeys {
                tuple_keys: vec![tuple_key],
            }),
            deletes: None,
            authorization_model_id: self.auth_model_id.clone(),
        };

        tracing::debug!(
            "Writing relationship: user={}, relation={}, object={}",
            user,
            relation,
            object
        );

        let url = format!("{}/stores/{}/write", self.endpoint, self.store_id);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("OpenFGA write request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "OpenFGA write failed with status {}: {}",
                status, error_text
            )));
        }

        // Invalidate cache for this object since permissions may have changed
        self.cache.invalidate_object(object_type, object_id).await;

        tracing::info!(
            "Successfully wrote relationship: user={}, relation={}, object={}",
            user,
            relation,
            object
        );

        Ok(())
    }

    /// Delete a relationship tuple from OpenFGA
    pub async fn delete_relationship(
        &self,
        user_id: Uuid,
        relation: &str,
        object_type: &str,
        object_id: &str,
    ) -> Result<()> {
        let user = format!("user:{}", user_id);
        let object = format!("{}:{}", object_type, object_id);

        let tuple_key = TupleKey {
            user: user.clone(),
            relation: relation.to_string(),
            object: object.clone(),
        };

        let request = WriteRequest {
            writes: None,
            deletes: Some(TupleKeys {
                tuple_keys: vec![tuple_key],
            }),
            authorization_model_id: self.auth_model_id.clone(),
        };

        tracing::debug!(
            "Deleting relationship: user={}, relation={}, object={}",
            user,
            relation,
            object
        );

        let url = format!("{}/stores/{}/write", self.endpoint, self.store_id);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("OpenFGA delete request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "OpenFGA delete failed with status {}: {}",
                status, error_text
            )));
        }

        // Invalidate cache for this object since permissions may have changed
        self.cache.invalidate_object(object_type, object_id).await;

        tracing::info!(
            "Successfully deleted relationship: user={}, relation={}, object={}",
            user,
            relation,
            object
        );

        Ok(())
    }

    /// List objects that a user has a specific relation to
    pub async fn list_objects(
        &self,
        user_id: Uuid,
        relation: &str,
        object_type: &str,
    ) -> Result<Vec<String>> {
        let user = format!("user:{}", user_id);

        let request = ListObjectsRequest {
            user: user.clone(),
            relation: relation.to_string(),
            object_type: object_type.to_string(),
            contextual_tuples: None,
        };

        tracing::debug!(
            "Listing objects: user={}, relation={}, object_type={}",
            user,
            relation,
            object_type
        );

        let url = format!("{}/stores/{}/list-objects", self.endpoint, self.store_id);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("OpenFGA list objects request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "OpenFGA list objects failed with status {}: {}",
                status, error_text
            )));
        }

        let list_response: ListObjectsResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse OpenFGA list objects response: {}", e)))?;

        tracing::debug!(
            "Listed {} objects for user={}, relation={}, object_type={}",
            list_response.objects.len(),
            user,
            relation,
            object_type
        );

        Ok(list_response.objects)
    }

    /// Health check for OpenFGA service
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/healthz", self.endpoint);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("OpenFGA health check failed: {}", e)))?;

        Ok(response.status().is_success())
    }

    /// Batch write multiple relationships
    pub async fn batch_write_relationships(&self, relationships: Vec<(Uuid, &str, &str, &str)>) -> Result<()> {
        if relationships.is_empty() {
            return Ok(());
        }

        // Clone the relationships for cache invalidation before consuming them
        let objects_to_invalidate: Vec<(String, String)> = relationships
            .iter()
            .map(|(_, _, object_type, object_id)| (object_type.to_string(), object_id.to_string()))
            .collect();

        let tuple_keys: Vec<TupleKey> = relationships
            .into_iter()
            .map(|(user_id, relation, object_type, object_id)| TupleKey {
                user: format!("user:{}", user_id),
                relation: relation.to_string(),
                object: format!("{}:{}", object_type, object_id),
            })
            .collect();

        let request = WriteRequest {
            writes: Some(TupleKeys { tuple_keys }),
            deletes: None,
            authorization_model_id: self.auth_model_id.clone(),
        };

        let url = format!("{}/stores/{}/write", self.endpoint, self.store_id);

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("OpenFGA batch write failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "OpenFGA batch write failed with status {}: {}",
                status, error_text
            )));
        }

        tracing::info!("Successfully wrote {} relationships in batch", request.writes.as_ref().unwrap().tuple_keys.len());

        // Invalidate cache for all affected objects
        for (object_type, object_id) in objects_to_invalidate {
            self.cache.invalidate_object(&object_type, &object_id).await;
        }

        Ok(())
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> crate::auth::cache::CacheStats {
        self.cache.stats().await
    }

    /// Clear permission cache
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }

    /// Invalidate cache for a specific user
    pub async fn invalidate_user_cache(&self, user_id: Uuid) {
        self.cache.invalidate_user(user_id).await;
    }

    /// Invalidate cache for a specific object
    pub async fn invalidate_object_cache(&self, object_type: &str, object_id: &str) {
        self.cache.invalidate_object(object_type, object_id).await;
    }
}

/// Helper functions for common authorization patterns

/// Check if a user can read a resource
pub async fn can_read(
    openfga: &OpenFgaService,
    user_id: Uuid,
    object_type: &str,
    object_id: &str,
) -> Result<bool> {
    let result = openfga
        .check_permission(user_id, "viewer", object_type, object_id)
        .await?;
    Ok(result.allowed)
}

/// Check if a user can write to a resource
pub async fn can_write(
    openfga: &OpenFgaService,
    user_id: Uuid,
    object_type: &str,
    object_id: &str,
) -> Result<bool> {
    let result = openfga
        .check_permission(user_id, "editor", object_type, object_id)
        .await?;
    Ok(result.allowed)
}

/// Check if a user owns a resource
pub async fn is_owner(
    openfga: &OpenFgaService,
    user_id: Uuid,
    object_type: &str,
    object_id: &str,
) -> Result<bool> {
    let result = openfga
        .check_permission(user_id, "owner", object_type, object_id)
        .await?;
    Ok(result.allowed)
}
