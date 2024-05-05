use pnet::util::MacAddr;
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};
use tracing::{debug, error};

use super::RepositoryError;

pub trait AllowedMacRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn contains(&self, address: &MacAddr) -> Result<bool, RepositoryError>;
    fn getall(&self) -> Result<Vec<MacAddr>, RepositoryError>;
    fn add(&self, address: MacAddr) -> Result<MacAddr, RepositoryError>;
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

    fn add(&self, address: MacAddr) -> Result<MacAddr, RepositoryError> {
        debug!("MAC address putted to AllowedMacRepository: {:?}", address);
        if let Ok(mut store) = self.store.write() {
            store.insert(address);
            Ok(address)
        } else {
            error!("Repository Error");
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

#[cfg(test)]
mod test {
    use pnet::util::MacAddr;

    use super::{AllowedMacRepository, AllowedMacRepositoryForMemory};

    #[test]
    fn allowedmac_repo_crd_scenario() {
        let addrs = vec![
            MacAddr::new(0x02, 0x00, 0x00, 0x00, 0x0f, 0x01),
            MacAddr::new(0x02, 0x00, 0x00, 0x00, 0x0f, 0x02),
            MacAddr::new(0x02, 0x00, 0x00, 0x00, 0x0f, 0x03),
        ];
        let repo = AllowedMacRepositoryForMemory::new();
        // add scenario (twice)
        for _ in 0..2 {
            for addr in addrs.iter() {
                repo.add(addr.clone()).expect("SyncFailed");
            }
        }
        // get scenario
        let mut repo_content = repo.getall().unwrap();
        repo_content.sort();
        assert_eq!(repo_content, addrs);
        assert!(repo_content.contains(addrs.get(0).unwrap()));
        assert!(!repo_content.contains(&MacAddr::new(2, 0, 0, 0, 0, 0)));
        // remove scenario
        let removing_addr = &addrs.get(0).unwrap();
        repo.remove(&removing_addr).expect("SyncErr");
        assert!(!repo.contains(removing_addr).expect("SyncErr"));
        // clear scenario
        repo.clear().expect("SyncErr");
        let repo_size = repo.getall().expect("SyncErr").len();
        assert_eq!(repo_size, 0);
    }
}
