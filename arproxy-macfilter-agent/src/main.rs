use std::str::FromStr;

use pnet::util::MacAddr;
use repositories::{AllowedMacRepository, AllowedMacRepositoryForMemory};
use tracing::debug;

mod config;
mod networks;
mod repositories;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // let config = config::load_config(filepath);
    let mut allowed_mac_repo = repositories::AllowedMacRepositoryForMemory::new();
    let mut arplog_repo = repositories::ArpLogRepositoryForMemory::new();
    allowed_mac_repo
        .put(MacAddr::from_str("02:00:00:00:00:01").unwrap())
        .unwrap();
    allowed_mac_repo.put(MacAddr::zero()).unwrap();
    allowed_mac_repo.put(MacAddr::broadcast()).unwrap();
    allowed_mac_repo.put(MacAddr::broadcast()).unwrap();
    println!("{:?}", allowed_mac_repo.getall())
}
