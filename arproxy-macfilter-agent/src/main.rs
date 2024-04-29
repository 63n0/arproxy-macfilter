mod config;
mod repositories;


use pnet::util::MacAddr;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

#[tokio::main]
async fn main() {
    let _seen_macs: HashMap<MacAddr, SystemTime> = HashMap::new();
    let config: config::Config = serde_json::from_str(
        r#"
    {
        "interface": "veth2",
        "arp_proxy_config": {
            "proxy_allowed_macs": true,
            "arp_reply_interval": 1,
            "arp_reply_duration": 10
        }
    }
    "#,
    )
    .unwrap();

    println!("{:?}", config);

    let seen_mac_repo = SeenMacRepository::new();
    seen_mac_repo.put(MacAddr::zero());
    seen_mac_repo.put(MacAddr::broadcast());

    let ttl = Duration::new(3, 0);
    for seen_mac in seen_mac_repo.clone().all_and_clean(ttl).into_iter() {
        println!("{:?}", seen_mac);
    }

    for _ in 0..3 {
        tokio::time::sleep(Duration::new(1, 0)).await;
        seen_mac_repo.put(MacAddr::broadcast());
    }

    for seen_mac in seen_mac_repo.clone().all_and_clean(ttl).into_iter() {
        println!("{:?}", seen_mac);
    }
}

#[derive(Debug, Clone)]
struct SeenMac {
    pub address: MacAddr,
    pub last_seen: SystemTime,
}

impl SeenMac {
    pub fn new(address: MacAddr) -> Self {
        Self {
            address,
            last_seen: SystemTime::now(),
        }
    }
}

type SeenMacs = HashMap<MacAddr, SeenMac>;
#[derive(Debug, Clone)]
struct SeenMacRepository {
    store: Arc<Mutex<SeenMacs>>,
}

impl SeenMacRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }

    /// 全てのエントリを取得し、超過したものは削除する
    pub fn all_and_clean(self, duration: Duration) -> Vec<SeenMac> {
        let mut hm = self.store.lock().unwrap();
        let mut result: Vec<SeenMac> = Vec::new();
        let mut remove_targets: Vec<MacAddr> = Vec::new();
        for (mac, seen_mac) in hm.iter() {
            if seen_mac.last_seen.elapsed().unwrap() > duration {
                remove_targets.push(mac.clone())
            } else {
                result.push(seen_mac.clone());
            }
        }
        for i in remove_targets {
            hm.remove(&i);
        }
        result
    }

    /// 新しいエントリを追加または上書きする
    pub fn put(&self, address: MacAddr) {
        let mut hm = self.store.lock().unwrap();
        hm.insert(address, SeenMac::new(address));
    }
}
