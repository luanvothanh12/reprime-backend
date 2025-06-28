use crate::auth::models::{AuthContext, Claims};
use crate::config::Config;
use crate::errors::{AppError, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration_hours: u64,
}

impl JwtService {
    pub fn new(config: &Config) -> Self {
        let secret = config.auth.jwt_secret.as_bytes();
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            expiration_hours: config.auth.jwt_expiration_hours,
        }
    }

    /// Generate a JWT token for a user
    pub fn generate_token(
        &self,
        user_id: Uuid,
        email: String,
        username: String,
        roles: Vec<String>,
    ) -> Result<String> {
        let now = Utc::now();
        let expiration = now + Duration::hours(self.expiration_hours as i64);

        let claims = Claims {
            sub: user_id.to_string(),
            email,
            username,
            roles,
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Authentication(format!("Failed to generate token: {}", e)))
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AppError::Authentication(format!("Invalid token: {}", e)))
    }

    /// Extract auth context from token
    pub fn extract_auth_context(&self, token: &str) -> Result<AuthContext> {
        let claims = self.validate_token(token)?;
        
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

        Ok(AuthContext {
            user_id,
            email: claims.email,
            username: claims.username,
            roles: claims.roles,
        })
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(auth_header: &str) -> Result<&str> {
        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Authentication(
                "Invalid authorization header format".to_string(),
            ));
        }

        Ok(&auth_header[7..]) // Remove "Bearer " prefix
    }

    /// Check if user has a required role
    pub fn has_role(auth_context: &AuthContext, required_role: &str) -> bool {
        auth_context.roles.contains(&required_role.to_string())
    }

    /// Check if a user has any of the required roles
    pub fn has_any_role(auth_context: &AuthContext, required_roles: &[&str]) -> bool {
        required_roles
            .iter()
            .any(|role| auth_context.roles.contains(&role.to_string()))
    }
}
