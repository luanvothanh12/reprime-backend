pub mod user;

use sqlx::PgPool;
use std::sync::Arc;

pub use user::UserRepository;

#[derive(Clone)]
pub struct Repositories {
    pub user: UserRepository,
}

impl Repositories {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            user: UserRepository::new(pool),
        }
    }
}
