pub mod allowed_mac;
pub mod arplog;
pub mod config;

#[derive(Debug, PartialEq, Eq,thiserror::Error)]
pub enum RepositoryError {
    #[error("Failed to get resource")]
    SyncFailed,
    #[error("Resource not found")]
    NotFound,
}
