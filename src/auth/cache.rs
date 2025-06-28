use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// In-memory cache for OpenFGA permission checks
#[derive(Debug)]
pub struct PermissionCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry<bool>>>>,
    default_ttl: Duration,
    max_entries: usize,
}

impl PermissionCache {
    pub fn new(default_ttl: Duration, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_entries,
        }
    }

    /// Generate cache key for permission check
    fn cache_key(user_id: Uuid, relation: &str, object_type: &str, object_id: &str) -> String {
        format!("{}:{}:{}:{}", user_id, relation, object_type, object_id)
    }

    /// Get cached permission result
    pub async fn get(
        &self,
        user_id: Uuid,
        relation: &str,
        object_type: &str,
        object_id: &str,
    ) -> Option<bool> {
        let key = Self::cache_key(user_id, relation, object_type, object_id);
        let cache = self.cache.read().await;
        
        if let Some(entry) = cache.get(&key) {
            if !entry.is_expired() {
                tracing::debug!("Cache hit for permission check: {}", key);
                return Some(entry.value);
            }
        }
        
        tracing::debug!("Cache miss for permission check: {}", key);
        None
    }

    /// Set cached permission result
    pub async fn set(
        &self,
        user_id: Uuid,
        relation: &str,
        object_type: &str,
        object_id: &str,
        allowed: bool,
    ) {
        self.set_with_ttl(user_id, relation, object_type, object_id, allowed, self.default_ttl)
            .await;
    }

    /// Set cached permission result with custom TTL
    pub async fn set_with_ttl(
        &self,
        user_id: Uuid,
        relation: &str,
        object_type: &str,
        object_id: &str,
        allowed: bool,
        ttl: Duration,
    ) {
        let key = Self::cache_key(user_id, relation, object_type, object_id);
        let entry = CacheEntry::new(allowed, ttl);
        
        let mut cache = self.cache.write().await;
        
        // Evict expired entries if cache is full
        if cache.len() >= self.max_entries {
            self.evict_expired(&mut cache).await;
            
            // If still full, remove oldest entries (simple LRU approximation)
            if cache.len() >= self.max_entries {
                let keys_to_remove: Vec<String> = cache
                    .keys()
                    .take(cache.len() - self.max_entries + 1)
                    .cloned()
                    .collect();
                
                for key_to_remove in keys_to_remove {
                    cache.remove(&key_to_remove);
                }
            }
        }
        
        cache.insert(key.clone(), entry);
        tracing::debug!("Cached permission result: {} = {}", key, allowed);
    }

    /// Invalidate cache entry
    pub async fn invalidate(
        &self,
        user_id: Uuid,
        relation: &str,
        object_type: &str,
        object_id: &str,
    ) {
        let key = Self::cache_key(user_id, relation, object_type, object_id);
        let mut cache = self.cache.write().await;
        cache.remove(&key);
        tracing::debug!("Invalidated cache entry: {}", key);
    }

    /// Invalidate all cache entries for a user
    pub async fn invalidate_user(&self, user_id: Uuid) {
        let user_prefix = format!("{}:", user_id);
        let mut cache = self.cache.write().await;
        
        let keys_to_remove: Vec<String> = cache
            .keys()
            .filter(|key| key.starts_with(&user_prefix))
            .cloned()
            .collect();
        
        for key in keys_to_remove {
            cache.remove(&key);
        }
        
        tracing::debug!("Invalidated all cache entries for user: {}", user_id);
    }

    /// Invalidate all cache entries for an object
    pub async fn invalidate_object(&self, object_type: &str, object_id: &str) {
        let object_suffix = format!(":{}:{}", object_type, object_id);
        let mut cache = self.cache.write().await;
        
        let keys_to_remove: Vec<String> = cache
            .keys()
            .filter(|key| key.ends_with(&object_suffix))
            .cloned()
            .collect();
        
        for key in keys_to_remove {
            cache.remove(&key);
        }
        
        tracing::debug!("Invalidated all cache entries for object: {}:{}", object_type, object_id);
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let count = cache.len();
        cache.clear();
        tracing::info!("Cleared {} cache entries", count);
    }

    /// Evict expired entries
    async fn evict_expired(&self, cache: &mut HashMap<String, CacheEntry<bool>>) {
        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in keys_to_remove {
            cache.remove(&key);
        }
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let total_entries = cache.len();
        let expired_entries = cache.values().filter(|entry| entry.is_expired()).count();
        
        CacheStats {
            total_entries,
            expired_entries,
            active_entries: total_entries - expired_entries,
            max_entries: self.max_entries,
            default_ttl: self.default_ttl,
        }
    }

    /// Background task to periodically clean up expired entries
    pub async fn cleanup_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60)); // Clean up every minute
        
        loop {
            interval.tick().await;
            
            let mut cache = self.cache.write().await;
            let initial_count = cache.len();
            self.evict_expired(&mut cache).await;
            let final_count = cache.len();
            
            if initial_count > final_count {
                tracing::debug!(
                    "Cache cleanup: removed {} expired entries ({} -> {})",
                    initial_count - final_count,
                    initial_count,
                    final_count
                );
            }
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
    pub max_entries: usize,
    pub default_ttl: Duration,
}

impl Default for PermissionCache {
    fn default() -> Self {
        Self::new(
            Duration::from_secs(300), // 5 minutes default TTL
            10000,                    // 10k max entries
        )
    }
}
