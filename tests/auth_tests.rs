use reprime_backend::auth::{
    jwt::JwtService,
    models::{AuthContext, Claims},
};
use reprime_backend::config::Config;
use uuid::Uuid;

#[tokio::test]
async fn test_jwt_token_generation_and_validation() {
    let config = Config::default();
    let jwt_service = JwtService::new(&config);

    let user_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let username = "testuser".to_string();
    let roles = vec!["user".to_string(), "admin".to_string()];

    // Generate token
    let token = jwt_service
        .generate_token(user_id, email.clone(), username.clone(), roles.clone())
        .expect("Failed to generate token");

    assert!(!token.is_empty());

    // Validate token
    let claims = jwt_service
        .validate_token(&token)
        .expect("Failed to validate token");

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.email, email);
    assert_eq!(claims.username, username);
    assert_eq!(claims.roles, roles);

    // Extract auth context
    let auth_context = jwt_service
        .extract_auth_context(&token)
        .expect("Failed to extract auth context");

    assert_eq!(auth_context.user_id, user_id);
    assert_eq!(auth_context.email, email);
    assert_eq!(auth_context.username, username);
    assert_eq!(auth_context.roles, roles);
}

#[tokio::test]
async fn test_jwt_token_expiration() {
    // Create a token that expires in the past
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, Header};

    let config = Config::default();
    let jwt_service = JwtService::new(&config);

    // Create an expired token manually
    let now = Utc::now();
    let expired_time = now - Duration::hours(1); // 1 hour ago

    let claims = reprime_backend::auth::models::Claims {
        sub: Uuid::new_v4().to_string(),
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        roles: vec!["user".to_string()],
        exp: expired_time.timestamp() as usize,
        iat: expired_time.timestamp() as usize,
    };

    let secret = config.auth.jwt_secret.as_bytes();
    let encoding_key = jsonwebtoken::EncodingKey::from_secret(secret);
    let expired_token = encode(&Header::default(), &claims, &encoding_key)
        .expect("Failed to create expired token");

    // Try to validate expired token
    let result = jwt_service.validate_token(&expired_token);
    assert!(result.is_err());
}

#[test]
fn test_role_checking() {
    let auth_context = AuthContext {
        user_id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        roles: vec!["user".to_string(), "admin".to_string()],
    };

    // Test has_role
    assert!(JwtService::has_role(&auth_context, "user"));
    assert!(JwtService::has_role(&auth_context, "admin"));
    assert!(!JwtService::has_role(&auth_context, "moderator"));

    // Test has_any_role
    assert!(JwtService::has_any_role(&auth_context, &["user"]));
    assert!(JwtService::has_any_role(&auth_context, &["admin"]));
    assert!(JwtService::has_any_role(&auth_context, &["user", "moderator"]));
    assert!(!JwtService::has_any_role(&auth_context, &["moderator", "guest"]));
}

#[test]
fn test_token_header_extraction() {
    // Valid Bearer token
    let auth_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.token";
    let token = JwtService::extract_token_from_header(auth_header)
        .expect("Failed to extract token");
    assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.token");

    // Invalid header format
    let invalid_header = "Basic dGVzdDp0ZXN0";
    let result = JwtService::extract_token_from_header(invalid_header);
    assert!(result.is_err());

    // Missing Bearer prefix
    let no_bearer = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.token";
    let result = JwtService::extract_token_from_header(no_bearer);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_openfga_service_creation() {
    let config = Config::default();
    let openfga_service = reprime_backend::auth::openfga::OpenFgaService::new(&config)
        .await
        .expect("Failed to create OpenFGA service");

    // Test health check (this will fail if no OpenFGA server is running, which is expected)
    let health_result = openfga_service.health_check().await;

    // We expect this to fail in tests since no OpenFGA server is running
    assert!(health_result.is_err() || !health_result.unwrap());

    // Test cache stats
    let stats = openfga_service.cache_stats().await;
    assert_eq!(stats.total_entries, 0);
    assert_eq!(stats.active_entries, 0);
}

#[tokio::test]
async fn test_openfga_cache_operations() {
    let config = Config::default();
    let openfga_service = reprime_backend::auth::openfga::OpenFgaService::new(&config)
        .await
        .expect("Failed to create OpenFGA service");

    let user_id = Uuid::new_v4();

    // Test cache operations without making network calls
    openfga_service.clear_cache().await;

    let stats = openfga_service.cache_stats().await;
    assert_eq!(stats.total_entries, 0);

    // Test cache invalidation methods (these don't make network calls)
    openfga_service.invalidate_user_cache(user_id).await;
    openfga_service.invalidate_object_cache("document", "doc-123").await;

    // Verify cache is still empty
    let stats_after = openfga_service.cache_stats().await;
    assert_eq!(stats_after.total_entries, 0);
}
