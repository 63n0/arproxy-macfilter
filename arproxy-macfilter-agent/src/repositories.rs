pub mod config;
pub mod arplog;
pub mod allowed_mac;
pub mod session;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Failed to get resource")]
    SyncFailed,
}