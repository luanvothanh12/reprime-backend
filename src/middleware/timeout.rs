use std::time::Duration;
use tower::timeout::TimeoutLayer;

pub fn timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(30))
}
