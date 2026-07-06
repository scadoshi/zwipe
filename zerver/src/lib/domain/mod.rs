pub mod auth;
pub mod card;
pub mod deck;
pub mod email;
#[cfg(feature = "zerver")]
pub mod health;
pub mod metrics;
pub mod user;

/// Boxed future returned by the `ErasedXService` twins (see each domain's
/// ports). One alias so the erased signatures stay readable.
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
