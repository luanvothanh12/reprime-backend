pub mod user;

use crate::database::InstrumentedDatabase;
use std::sync::Arc;

pub use user::UserRepository;

#[derive(Clone)]
pub struct Repositories {
    pub user: UserRepository,
}

impl Repositories {

    pub fn new(instrumented_db: Arc<InstrumentedDatabase>) -> Self {
        Self {
            user: UserRepository::new(instrumented_db),
        }
    }
}
