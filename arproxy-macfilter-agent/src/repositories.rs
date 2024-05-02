use std::{
    collections::{HashMap, HashSet}, fs::remove_dir, net::Ipv4Addr, sync::{Arc, LockResult, Mutex, RwLock, RwLockWriteGuard}, time::{Duration, SystemTime}
};

use axum::http::Response;
use pnet::util::MacAddr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Failed to get resource")]
    SyncFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ArpLogKey{
    source_mac: MacAddr,
    source_ip: Ipv4Addr,
    // target_mac: MacAddr, ARP Request target MAC address always all-zero
    target_ip: Ipv4Addr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpLog {
    source_mac: MacAddr,
    source_ip: Ipv4Addr,
    target_mac: MacAddr,
    target_ip: Ipv4Addr,
    last_seen: SystemTime,
}

impl ArpLog{
    fn new(source_mac:MacAddr, source_ip:Ipv4Addr, target_mac:MacAddr, target_ip:Ipv4Addr) -> Self {
        Self {
            source_mac, source_ip, target_ip, target_mac, 
            last_seen: SystemTime::now(),
        }
    }

    fn extract_key(&self) -> ArpLogKey {
        ArpLogKey {
            source_mac: self.source_mac,
            source_ip: self.source_ip,
            target_ip: self.target_ip, 
        }
    }
}

trait ArpLogRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    /// ArpLogを挿入またはlast_seenを更新する
    fn put(&self, arplog: ArpLog) -> Result<(), RepositoryError>;
    /// 全てのArpLogを取得し、同時にdurationを超過したものは削除する
    fn getall_autoclear(&self, duration: Duration) -> Result<Vec<ArpLog>, RepositoryError>;
    fn getall_without_autoclear(&self) -> Result<Vec<ArpLog>, RepositoryError>;
    fn clear(&self) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct ArpLogRepositoryForMemory {
    store: Arc<RwLock<HashMap<ArpLogKey, ArpLog>>>,
}
impl ArpLogRepositoryForMemory {
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }
}

impl ArpLogRepository for ArpLogRepositoryForMemory {
    fn put (&self, arplog:ArpLog) -> Result<(), RepositoryError> {
        if let Ok(mut store) = self.store.write() {
            store.insert(arplog.extract_key(), arplog);
            Ok(())
        } else {
            Err(RepositoryError::SyncFailed)
        }
    }

    fn getall_autoclear(&self, duration: Duration) -> Result<Vec<ArpLog>, RepositoryError> {
        if let Ok(mut store) = self.store.write() {
            let mut result = Vec::new();
            let mut removing = Vec::new();
            for (key, arplog) in store.iter() {
                if (arplog.last_seen.elapsed().unwrap_or(duration) <= duration) {
                    result.push(arplog.clone());
                } else {
                    removing.push(key.clone());
                }
            }
            for e in removing{
                store.remove(&e);
            }
            Ok(result)
        } else {
            Err(RepositoryError::SyncFailed)
        }
    }

    fn getall_without_autoclear(&self) -> Result<Vec<ArpLog>, RepositoryError> {
        if let Ok(store) = self.store.read() {
            let mut result = Vec::new();
            for (_, arplog) in store.iter(){
                result.push(arplog.clone());
            }
            Ok(result)
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



pub trait AllowedMacRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn contains(&self, address:MacAddr) -> Result<bool, RepositoryError>;
    fn getall(&self) -> Result<Vec<MacAddr>, RepositoryError>;
    fn put(&self, address:MacAddr) -> Result<(), RepositoryError>;
    fn remove(&self, address:MacAddr) -> Result<(), RepositoryError>;
    fn clear(&self) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct AllowedMacRepositoryForMemory {
    store:Arc<RwLock<HashSet<MacAddr>>>,
}

impl AllowedMacRepositoryForMemory {
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }
}


impl AllowedMacRepository for AllowedMacRepositoryForMemory {
    fn contains(&self, address:MacAddr) -> Result<bool, RepositoryError> {
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

    fn put(&self, address:MacAddr) -> Result<(), RepositoryError> {
        if let Ok(mut store) = self.store.write() {
            store.insert(address);
            Ok(())
        } else {
            Err(RepositoryError::SyncFailed)
        }
    }

    fn remove(&self, address:MacAddr) -> Result<(), RepositoryError> {
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