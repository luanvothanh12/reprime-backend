pub mod health;
pub mod user;

use crate::services::Services;
use std::sync::Arc;

pub use health::{health_check, HealthResponse};
pub use user::{UserHandlers, create_user, get_user, get_users, update_user, delete_user};

#[derive(Clone)]
pub struct Handlers {
    pub user: UserHandlers,
}

impl Handlers {
    pub fn new(services: Arc<Services>) -> Self {
        Self {
            user: UserHandlers::new(services.clone()),
        }
    }
}
