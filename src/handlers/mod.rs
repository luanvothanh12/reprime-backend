pub mod health;
pub mod metrics;
pub mod user;

use crate::client::HttpClientService;
use crate::services::Services;
use std::sync::Arc;

pub use health::{health_check, HealthResponse};
pub use metrics::metrics_handler;
pub use user::{UserHandlers, create_user, get_user, get_users, update_user, delete_user};

#[derive(Clone)]
pub struct Handlers {
    pub user: UserHandlers,
}

impl Handlers {
    pub fn new(services: Arc<Services>, http_client_service: Arc<HttpClientService>) -> Self {
        Self {
            user: UserHandlers::new(services.clone()),
        }
    }
}
