use crate::config;
use std::sync::{Arc, RwLock};

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