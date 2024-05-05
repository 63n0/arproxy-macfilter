pub mod allowed_mac;
pub mod arplog;
pub mod config;
pub mod session;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Failed to get resource")]
    SyncFailed,
}
