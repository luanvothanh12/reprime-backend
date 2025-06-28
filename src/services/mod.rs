pub mod auth;
pub mod user;

use crate::repositories::Repositories;
use std::sync::Arc;

pub use auth::AuthService;
pub use user::UserService;

#[derive(Clone)]
pub struct Services {
    pub user: UserService,
    pub auth: AuthService,
}

impl Services {
    pub fn new(
        repositories: Arc<Repositories>,
        jwt_service: Arc<crate::auth::jwt::JwtService>,
        openfga_service: Arc<crate::auth::openfga::OpenFgaService>,
    ) -> Self {
        let user_service = Arc::new(UserService::new(repositories.clone()));

        Self {
            user: (*user_service).clone(),
            auth: AuthService::new(
                repositories,
                user_service,
                jwt_service,
                openfga_service,
            ),
        }
    }
}
