use std::future::Future;

use crate::domain::health::models::HealthCheckFailed;

pub trait HealthRepository: Clone + Send + Sync + 'static {
    fn check_database(&self) -> impl Future<Output = Result<(), HealthCheckFailed>> + Send;
}

pub trait HealthService: Clone + Send + Sync + 'static {
    fn check_database(&self) -> impl Future<Output = Result<(), HealthCheckFailed>> + Send;
}
