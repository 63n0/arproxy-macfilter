use std::net::Ipv4Addr;

use pnet::datalink::{Channel, NetworkInterface};
use pnet::packet::arp::*;
use pnet::packet::ethernet::*;
use pnet::packet::{MutablePacket, Packet};
use pnet::util::MacAddr;

// https://github.com/Dineshs91/send-arp/blob/master/src/main.rs
fn send_arp_packet(
    interface: NetworkInterface,
    source_mac: MacAddr,
    source_ip: Ipv4Addr,
    target_mac: MacAddr,
    target_ip: Ipv4Addr,
    arp_operation: ArpOperation,
) {
    let (mut tx, _rx) = match pnet::datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };

    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_frame =
        MutableEthernetPacket::new(&mut ethernet_buffer).expect("Packet Creation Failed");

    ethernet_frame.set_destination(target_mac);
    ethernet_frame.set_source(source_mac);
    ethernet_frame.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_frame = MutableArpPacket::new(&mut arp_buffer).unwrap();
    arp_frame.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_frame.set_protocol_type(EtherTypes::Ipv4);
    arp_frame.set_hw_addr_len(6);
    arp_frame.set_proto_addr_len(4);
    arp_frame.set_operation(arp_operation);
    arp_frame.set_sender_hw_addr(source_mac);
    arp_frame.set_sender_proto_addr(source_ip);
    arp_frame.set_target_hw_addr(target_mac);
    arp_frame.set_target_proto_addr(target_ip);

    ethernet_frame.set_payload(arp_frame.packet_mut());

    tx.send_to(&ethernet_frame.packet(), Some(interface));
}

fn main() {
    let iface_name = "lo".to_string();
    let interfaces = pnet::datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == iface_name)
        .expect("[Error] Interface name not found");
    let source_mac: MacAddr = interface.mac.unwrap_or(MacAddr::zero());
    let source_ip = Ipv4Addr::new(192, 0, 2, 1);
    let target_mac: MacAddr = MacAddr::broadcast();
    let target_ip = Ipv4Addr::new(192, 0, 2, 3);
    send_arp_packet(
        interface,
        source_mac,
        source_ip,
        target_mac,
        target_ip,
        ArpOperations::Request,
    );
}


// use pnet::util::MacAddr;
// use std::{
//     collections::HashMap,
//     sync::{Arc, Mutex},
//     time::{Duration, SystemTime},
// };

// #[tokio::main]
// async fn main() {
//     let _seen_macs: HashMap<MacAddr, SystemTime> = HashMap::new();
//     let config: config::Config = serde_json::from_str(
//         r#"
//     {
//         "interface": "veth2",
//         "arp_proxy_config": {
//             "proxy_allowed_macs": true,
//             "arp_reply_interval": 1,
//             "arp_reply_duration": 10
//         }
//     }
//     "#,
//     )
//     .unwrap();

//     println!("{:?}", config);

//     let seen_mac_repo = SeenMacRepository::new();
//     seen_mac_repo.put(MacAddr::zero());
//     seen_mac_repo.put(MacAddr::broadcast());

//     let ttl = Duration::new(3, 0);
//     for seen_mac in seen_mac_repo.clone().all_and_clean(ttl).into_iter() {
//         println!("{:?}", seen_mac);
//     }

//     for _ in 0..3 {
//         tokio::time::sleep(Duration::new(1, 0)).await;
//         seen_mac_repo.put(MacAddr::broadcast());
//     }

//     for seen_mac in seen_mac_repo.clone().all_and_clean(ttl).into_iter() {
//         println!("{:?}", seen_mac);
//     }
// }

// #[derive(Debug, Clone)]
// struct SeenMac {
//     pub address: MacAddr,
//     pub last_seen: SystemTime,
// }

// impl SeenMac {
//     pub fn new(address: MacAddr) -> Self {
//         Self {
//             address,
//             last_seen: SystemTime::now(),
//         }
//     }
// }

// type SeenMacs = HashMap<MacAddr, SeenMac>;
// #[derive(Debug, Clone)]
// struct SeenMacRepository {
//     store: Arc<Mutex<SeenMacs>>,
// }

// impl SeenMacRepository {
//     pub fn new() -> Self {
//         Self {
//             store: Arc::default(),
//         }
//     }

//     /// 全てのエントリを取得し、超過したものは削除する
//     pub fn all_and_clean(self, duration: Duration) -> Vec<SeenMac> {
//         let mut hm = self.store.lock().unwrap();
//         let mut result: Vec<SeenMac> = Vec::new();
//         let mut remove_targets: Vec<MacAddr> = Vec::new();
//         for (mac, seen_mac) in hm.iter() {
//             if seen_mac.last_seen.elapsed().unwrap() > duration {
//                 remove_targets.push(mac.clone())
//             } else {
//                 result.push(seen_mac.clone());
//             }
//         }
//         for i in remove_targets {
//             hm.remove(&i);
//         }
//         result
//     }

//     /// 新しいエントリを追加または上書きする
//     pub fn put(&self, address: MacAddr) {
//         let mut hm = self.store.lock().unwrap();
//         hm.insert(address, SeenMac::new(address));
//     }
// }
