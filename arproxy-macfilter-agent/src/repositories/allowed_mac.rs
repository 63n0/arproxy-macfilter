use pnet::util::MacAddr;
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};
use tracing::debug;

use super::RepositoryError;

pub trait AllowedMacRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn contains(&self, address: &MacAddr) -> Result<bool, RepositoryError>;
    fn getall(&self) -> Result<Vec<MacAddr>, RepositoryError>;
    fn put(&self, address: MacAddr) -> Result<(), RepositoryError>;
    fn remove(&self, address: &MacAddr) -> Result<(), RepositoryError>;
    fn clear(&self) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct AllowedMacRepositoryForMemory {
    store: Arc<RwLock<HashSet<MacAddr>>>,
}

impl AllowedMacRepositoryForMemory {
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }
}

impl AllowedMacRepository for AllowedMacRepositoryForMemory {
    fn contains(&self, address: &MacAddr) -> Result<bool, RepositoryError> {
        if let Ok(store) = self.store.read() {
            Ok(store.contains(&address))
        } else {
            Err(RepositoryError::SyncFailed)
        }
    }

    fn getall(&self) -> Result<Vec<MacAddr>, RepositoryError> {
        if let Ok(store) = self.store.read() {
            Ok(store.clone().into_iter().collect())
        } else {
            Err(RepositoryError::SyncFailed)
        }
    }

    fn put(&self, address: MacAddr) -> Result<(), RepositoryError> {
        debug!("MAC address putted to AllowedMacRepository: {:?}", address);
        if let Ok(mut store) = self.store.write() {
            store.insert(address);
            Ok(())
        } else {
            Err(RepositoryError::SyncFailed)
        }
    }

    fn remove(&self, address: &MacAddr) -> Result<(), RepositoryError> {
        if let Ok(mut store) = self.store.write() {
            store.remove(&address);
            Ok(())
        } else {
            Err(RepositoryError::SyncFailed)
        }
    }

    fn clear(&self) -> Result<(), RepositoryError> {
        if let Ok(mut store) = self.store.write() {
            store.clear();
            Ok(())
        } else {
            Err(RepositoryError::SyncFailed)
        }
    }
}
