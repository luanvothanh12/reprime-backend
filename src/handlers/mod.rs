pub mod health;
pub mod metrics;
pub mod user;

use crate::auth::handlers::AuthHandlers;
use crate::auth::jwt::JwtService;
use crate::auth::openfga::OpenFgaService;
use crate::services::Services;
use std::sync::Arc;

pub use health::{health_check, HealthResponse};
pub use metrics::metrics_handler;
pub use user::{UserHandlers, create_user, get_user, get_users, update_user, delete_user};

#[derive(Clone)]
pub struct Handlers {
    pub user: UserHandlers,
    pub auth: AuthHandlers,
}

impl Handlers {
    pub fn new(
        services: Arc<Services>,
        jwt_service: Arc<JwtService>,
        openfga_service: Arc<OpenFgaService>,
    ) -> Self {
        Self {
            user: UserHandlers::new(services.clone()),
            auth: AuthHandlers::new(services, jwt_service, openfga_service),
        }
    }
}
