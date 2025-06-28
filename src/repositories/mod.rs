pub mod auth;
pub mod user;

use crate::database::InstrumentedDatabase;
use std::sync::Arc;

pub use auth::AuthRepository;
pub use user::UserRepository;

#[derive(Clone)]
pub struct Repositories {
    pub user: UserRepository,
    pub auth: AuthRepository,
}

impl Repositories {
    pub fn new(instrumented_db: Arc<InstrumentedDatabase>) -> Self {
        Self {
            user: UserRepository::new(instrumented_db.clone()),
            auth: AuthRepository::new(instrumented_db),
        }
    }
}
