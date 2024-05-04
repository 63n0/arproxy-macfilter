use std::thread;

use pnet::datalink::Channel;
use repositories::{ArpLogRepository, ConfigRepository};
use tracing::{debug, info, trace};

mod config;
mod networks;
mod repositories;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config_str = r#"{
        "interface":"veth2",
        "arp_proxy_config": {
            "proxy_allowed_macs": false,
            "arp_reply_interval": 1,
            "arp_reply_duration": 10
        }
    }
    "#;
    let config: config::Config = serde_json::from_str(&config_str).unwrap();
    trace!("{:?}", config);
    let allowedmac_repo = repositories::AllowedMacRepositoryForMemory::new();
    let arplog_repo = repositories::ArpLogRepositoryForMemory::new();
    let config_repo = repositories::ConfigRepositoryForMemory::new(config);

    let iface_name = config_repo.get_config().interface.clone();
    let interfaces = pnet::datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == iface_name)
        .expect("[Error] Interface name not found");

    let packet_sender = networks::PacketSender::new(
        config_repo.clone(),
        allowedmac_repo.clone(),
        arplog_repo.clone(),
        interface.clone(),
    );
    let packet_listener = networks::PacketListener::new(
        config_repo.clone(),
        allowedmac_repo.clone(),
        arplog_repo.clone(),
        interface.clone(),
        packet_sender.clone(),
    );

    let t = thread::spawn(move || {
        packet_listener.listen()
        // for i in 0..200 {
        //     trace!("Packet Received: {}", i);
        //     match rx.next() {
        //         Ok(pkt) => packet_handler.handle_frame(pkt).unwrap(),
        //         Err(e) => panic!("Failed"),
        //     };
        // }
    });
    debug!("Listening Handler Spawned");
    packet_sender.send_loop().await;

    t.join();

    for arplog in arplog_repo.getall_without_autoclear().unwrap() {
        info!("{:?}", arplog);
    }

    // allowed_mac_repo
    //     .put(MacAddr::from_str("02:00:00:00:00:01").unwrap())
    //     .unwrap();
    // allowed_mac_repo.put(MacAddr::zero()).unwrap();
    // allowed_mac_repo.put(MacAddr::broadcast()).unwrap();
    // allowed_mac_repo.put(MacAddr::broadcast()).unwrap();
    // println!("{:?}", allowed_mac_repo.getall())
}
