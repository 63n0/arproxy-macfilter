use std::{
    collections::{HashMap, HashSet},
    net::Ipv4Addr,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use pnet::util::MacAddr;
use thiserror::Error;
use tracing::trace;

use crate::config::{self};

pub trait ConfigRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn get_config(&self) -> config::Config;
}

#[derive(Clone, Debug)]
pub struct ConfigRepositoryForMemory {
    store: Arc<RwLock<config::Config>>,
}

impl ConfigRepositoryForMemory {
    pub fn new(config: config::Config) -> Self {
        Self {
            store: Arc::new(RwLock::new(config)),
        }
    }
}

impl ConfigRepository for ConfigRepositoryForMemory {
    // うまく排他制御できてなさそうである。
    fn get_config(&self) -> config::Config {
        self.store.read().unwrap().clone()
    }
}

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
        trace!("MAC address putted to AllowedMacRepository: {:?}", address);
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

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Failed to get resource")]
    SyncFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpLog {
    pub sender_mac: MacAddr,
    pub sender_ip: Ipv4Addr,
    // pub target_mac: MacAddr, target mac address always all-zero
    pub target_ip: Ipv4Addr,
    pub last_seen: SystemTime,
}

impl ArpLog {
    pub fn new(sender_mac: MacAddr, sender_ip: Ipv4Addr, target_ip: Ipv4Addr) -> Self {
        Self {
            sender_mac,
            sender_ip,
            target_ip,
            last_seen: SystemTime::now(),
        }
    }
}

pub trait ArpLogRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    /// ArpLogを挿入またはlast_seenを更新する
    ///
    fn put(&self, arplog: ArpLog) -> Result<(), RepositoryError>;
    /// 全てのArpLogを取得し、同時にdurationを超過したものは削除する
    fn getall_autoclear(&self, duration: Duration) -> Result<Vec<ArpLog>, RepositoryError>;
    fn getall_without_autoclear(&self) -> Result<Vec<ArpLog>, RepositoryError>;
    fn clear(&self) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
struct ArpLogForMemory {
    pub sender_mac: MacAddr,
    pub sender_ip: Ipv4Addr,
    // pub target_mac: MacAddr, target mac address always all-zero
    // pub target_ips: Vec<(Ipv4Addr, SystemTime)>,
    pub target_ips: HashMap<Ipv4Addr, SystemTime>,
    pub last_seen: SystemTime,
}

impl ArpLogForMemory {
    fn to_arplogs_autoclear(&mut self, duration: Duration) -> Vec<ArpLog> {
        let mut result = Vec::new();
        let mut template = ArpLog {
            sender_mac: self.sender_mac,
            sender_ip: self.sender_ip,
            target_ip: Ipv4Addr::new(0, 0, 0, 0),
            last_seen: SystemTime::now(),
        };
        /* Vector Implementation
        let mut i = 0;
        while(i < self.target_ips.len()){
            let (tip, time)= self.target_ips[i];
            if(time.elapsed().unwrap() < duration){
                template.target_ip = tip.clone();
                template.last_seen = time.clone();
                result.push(template.clone());
                i += 1;
            } else {
                self.target_ips.remove(i);
            }
        }
         */
        let mut removing = Vec::new();
        for (tip, time) in self.target_ips.iter() {
            if time.elapsed().unwrap() < duration {
                template.target_ip = tip.clone();
                template.last_seen = time.clone();
                result.push(template.clone());
            } else {
                removing.push(tip.clone());
            }
        }

        for tip in removing {
            self.target_ips.remove(&tip);
        }
        result
    }

    fn to_arplog(&self) -> Vec<ArpLog> {
        let mut result = Vec::new();
        let mut template = ArpLog {
            sender_mac: self.sender_mac,
            sender_ip: self.sender_ip,
            target_ip: Ipv4Addr::new(0, 0, 0, 0),
            last_seen: SystemTime::now(),
        };
        for (tip, time) in self.target_ips.iter() {
            template.target_ip = tip.clone();
            template.last_seen = time.clone();
            result.push(template.clone());
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct ArpLogRepositoryForMemory {
    store: Arc<RwLock<HashMap<MacAddr, ArpLogForMemory>>>,
}

impl ArpLogRepositoryForMemory {
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }
}

impl ArpLogRepository for ArpLogRepositoryForMemory {
    fn put(&self, arplog: ArpLog) -> Result<(), RepositoryError> {
        trace!("ArpLog putted to ArpLogRepositoryForMemory: {:?}", arplog);
        if let Ok(mut store) = self.store.write() {
            if let Some(alfm) = store.get_mut(&arplog.sender_mac) {
                alfm.target_ips.insert(arplog.target_ip, arplog.last_seen);
                alfm.last_seen = arplog.last_seen;
            } else {
                let mut tipmap = HashMap::new();
                tipmap.insert(arplog.target_ip, arplog.last_seen);
                store.insert(
                    arplog.sender_mac,
                    ArpLogForMemory {
                        sender_mac: arplog.sender_mac,
                        sender_ip: arplog.sender_ip,
                        target_ips: tipmap,
                        last_seen: arplog.last_seen,
                    },
                );
            }

            Ok(())
        } else {
            Err(RepositoryError::SyncFailed)
        }
    }

    fn getall_autoclear(&self, duration: Duration) -> Result<Vec<ArpLog>, RepositoryError> {
        if let Ok(mut store) = self.store.write() {
            let mut result = Vec::new();
            let mut removing = Vec::new();

            for (smac, arplog) in store.iter_mut() {
                if arplog.last_seen.elapsed().unwrap_or(duration) <= duration {
                    result.append(&mut arplog.to_arplogs_autoclear(duration));
                } else {
                    removing.push(smac.clone());
                }
            }

            for e in removing {
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
            for (_, arplog) in store.iter() {
                result.append(&mut arplog.to_arplog());
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
