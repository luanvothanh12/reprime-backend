pub mod user;

use crate::repositories::Repositories;
use std::sync::Arc;

pub use user::UserService;

#[derive(Clone)]
pub struct Services {
    pub user: UserService,
}

impl Services {
    pub fn new(repositories: Arc<Repositories>) -> Self {
        Self {
            user: UserService::new(repositories.clone()),
        }
    }
}
