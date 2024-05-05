use std::time::{Duration, SystemTime};

use pnet::{
    datalink::{Channel, DataLinkReceiver, DataLinkSender, NetworkInterface},
    packet::{
        arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket},
        ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
        Packet,
    },
    util::MacAddr,
};
use tracing::{debug, trace};

use crate::repositories::{
    allowed_mac::AllowedMacRepository, arplog::ArpLog, arplog::ArpLogRepository,
    config::ConfigRepository,
};

#[derive(Debug)]
pub enum NetworkError {
    UnitSizeError(String),
}

pub struct PacketListener<C, M, A>
where
    C: ConfigRepository,
    M: AllowedMacRepository,
    A: ArpLogRepository,
{
    config_repo: C,
    allowedmac_repo: M,
    arplog_repo: A,
    interface: NetworkInterface,
    packet_sender: PacketSender<C, M, A>,
}

/*
pub struct PacketHandler<C, M, A>
where
    C: ConfigRepository,
    M: AllowedMacRepository,
    A: ArpLogRepository,
{
    config_repo: C,
    allowedmac_repo: M,
    arplog_repo: A,
}
*/

impl<C, M, A> PacketListener<C, M, A>
where
    C: ConfigRepository,
    M: AllowedMacRepository,
    A: ArpLogRepository,
{
    pub fn new(
        config_repo: C,
        allowedmac_repo: M,
        arplog_repo: A,
        interface: NetworkInterface,
        packet_sender: PacketSender<C, M, A>,
    ) -> Self {
        Self {
            config_repo,
            allowedmac_repo,
            arplog_repo,
            interface,
            packet_sender,
        }
    }

    pub fn listen(&self) {
        let (_, mut rx) = match pnet::datalink::channel(&self.interface, Default::default()) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("Unknown channel type"),
            Err(e) => panic!("Error happened {}", e),
        };

        loop {
            match rx.next() {
                Ok(pkt) => self.handle_frame(pkt).unwrap(),
                Err(_e) => panic!("Failed"),
            };
        }
    }

    fn handle_frame(&self, frame: &[u8]) -> Result<(), NetworkError> {
        if let Some(ethernet_frame) = EthernetPacket::new(frame) {
            self.handle_ethernet(&ethernet_frame)
        } else {
            Err(NetworkError::UnitSizeError(
                "less than minimal Ethernet frame size".to_string(),
            ))
        }
    }

    fn handle_ethernet(&self, frame: &EthernetPacket) -> Result<(), NetworkError> {
        if frame.get_source() == self.interface.mac.unwrap() {
            return Ok(());
        }
        if frame.get_ethertype() == EtherTypes::Arp {
            if let Some(arp_frame) = ArpPacket::new(frame.payload()) {
                self.handle_arp(&arp_frame)
            } else {
                Err(NetworkError::UnitSizeError(
                    "less than minimal ARP frame size".to_string(),
                ))
            }
        } else {
            Ok(())
        }
    }

    fn handle_arp(&self, frame: &ArpPacket) -> Result<(), NetworkError> {
        if frame.get_operation() == ArpOperations::Request {
            let arplog = ArpLog {
                sender_mac: frame.get_sender_hw_addr(),
                sender_ip: frame.get_sender_proto_addr(),
                target_ip: frame.get_target_proto_addr(),
                last_seen: SystemTime::now(),
            };
            trace!("ArpRequestReceived: {:?}", arplog);
            let proxy_config = self.config_repo.get_config().arp_proxy_config;
            if proxy_config.proxy_allowed_macs
                || !self.allowedmac_repo.contains(&arplog.sender_mac).unwrap()
            {
                self.arplog_repo.put(arplog.clone());
                // MUST implement fake arp reply
                self.packet_sender.send_spoofing_frame(arplog.clone());
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
pub struct PacketSender<C, M, A>
where
    C: ConfigRepository,
    M: AllowedMacRepository,
    A: ArpLogRepository,
{
    config_repo: C,
    allowedmac_repo: M,
    arplog_repo: A,
    interface: NetworkInterface,
    // _tx: Box<dyn DataLinkSender>,
}

impl<C, M, A> PacketSender<C, M, A>
where
    C: ConfigRepository,
    M: AllowedMacRepository,
    A: ArpLogRepository,
{
    pub fn new(
        config_repo: C,
        allowedmac_repo: M,
        arplog_repo: A,
        interface: NetworkInterface,
    ) -> Self {
        Self {
            config_repo,
            allowedmac_repo,
            arplog_repo,
            interface,
        }
    }

    pub async fn send_loop(&self) {
        let arplog_life = Duration::from_secs(
            self.config_repo
                .get_config()
                .arp_proxy_config
                .arp_reply_duration
                .into(),
        );
        let interval_secs = self
            .config_repo
            .get_config()
            .arp_proxy_config
            .arp_reply_interval;
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs.into()));
        loop {
            let time = SystemTime::now();
            let arplogs = self.arplog_repo.getall_autoclear(arplog_life);
            for arplog in arplogs.unwrap() {
                self.send_spoofing_frame(arplog);
            }
            interval.tick().await;
            debug!(
                "Sendloop: {:?} sec/loop",
                time.elapsed().unwrap().as_secs_f64()
            );
        }
    }
    fn construct_proxyarp_frames(&self, arplog: ArpLog) -> ([u8; 42], [u8; 42]) {
        // (smac, sip, tmac, tip, op)
        // frame1: 正規のARPリクエストに偽装したARP応答
        // frame2: Target IP に指定された機器のARPテーブルを書き換えるためのARP要求
        let sender_mac = self.interface.mac.unwrap();
        let mut ethernet_buffer1 = [0u8; 42];
        let mut ethernet_buffer2 = [0u8; 42];
        let mut ethernet_frame1 =
            MutableEthernetPacket::new(&mut ethernet_buffer1).expect("Packet Creation Failed");
        let mut ethernet_frame2 =
            MutableEthernetPacket::new(&mut ethernet_buffer2).expect("Packet Creation Failed");
        ethernet_frame1.set_source(sender_mac);
        ethernet_frame1.set_destination(arplog.sender_mac);
        ethernet_frame2.set_source(sender_mac);
        ethernet_frame2.set_destination(MacAddr::broadcast());

        ethernet_frame1.set_ethertype(EtherTypes::Arp);
        ethernet_frame2.set_ethertype(EtherTypes::Arp);

        let mut arp_buffer1 = [0u8; 28];
        let mut arp_buffer2 = [0u8; 28];
        let mut arp_frame1 =
            MutableArpPacket::new(&mut arp_buffer1).expect("Packet Creation Failed");
        let mut arp_frame2 =
            MutableArpPacket::new(&mut arp_buffer2).expect("Packet Creation Failed");
        arp_frame1.set_hardware_type(ArpHardwareTypes::Ethernet);
        arp_frame2.set_hardware_type(ArpHardwareTypes::Ethernet);
        arp_frame1.set_protocol_type(EtherTypes::Ipv4);
        arp_frame2.set_protocol_type(EtherTypes::Ipv4);
        arp_frame1.set_hw_addr_len(6);
        arp_frame2.set_hw_addr_len(6);
        arp_frame1.set_proto_addr_len(4);
        arp_frame2.set_proto_addr_len(4);

        arp_frame1.set_sender_hw_addr(sender_mac);
        arp_frame1.set_sender_proto_addr(arplog.target_ip);
        arp_frame1.set_target_hw_addr(arplog.sender_mac);
        arp_frame1.set_target_proto_addr(arplog.sender_ip);
        arp_frame1.set_operation(ArpOperations::Reply);

        arp_frame2.set_sender_hw_addr(sender_mac);
        arp_frame2.set_sender_proto_addr(arplog.sender_ip);
        arp_frame2.set_target_hw_addr(MacAddr::zero());
        arp_frame2.set_target_proto_addr(arplog.target_ip);
        arp_frame2.set_operation(ArpOperations::Request);

        ethernet_frame1.set_payload(arp_frame1.packet());
        ethernet_frame2.set_payload(arp_frame2.packet());

        (ethernet_buffer1, ethernet_buffer2)
    }

    pub fn send_spoofing_frame(&self, arplog: ArpLog) {
        let (mut tx, _rx) = match pnet::datalink::channel(&self.interface, Default::default()) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("Unknown channel type"),
            Err(e) => panic!("Error happened {}", e),
        };
        let (raw_frame1, raw_frame2) = self.construct_proxyarp_frames(arplog);
        tx.send_to(&raw_frame1, Some(self.interface.clone()));
        tx.send_to(&raw_frame2, Some(self.interface.clone()));
    }
}
/*
// 本当はEthernetPacketを返したいが、これがバッファを内部的に参照しているためうまくいかない。rustは参照を返せない。
/// ArpLogに対応してARPプロキシのために送信されるARPフレームを生成する
fn construct_proxyarp_frames(arplog: ArpLog, sender_mac: MacAddr) -> ([u8; 42], [u8; 42]) {
    // (smac, sip, tmac, tip, op)
    // frame1: 正規のARPリクエストに偽装したARP応答
    // frame2: Target IP に指定された機器のARPテーブルを書き換えるためのARP要求
    let mut ethernet_buffer1 = [0u8; 42];
    let mut ethernet_buffer2 = [0u8; 42];
    let mut ethernet_frame1 =
        MutableEthernetPacket::new(&mut ethernet_buffer1).expect("Packet Creation Failed");
    let mut ethernet_frame2 =
        MutableEthernetPacket::new(&mut ethernet_buffer2).expect("Packet Creation Failed");
    ethernet_frame1.set_source(sender_mac);
    ethernet_frame1.set_destination(arplog.sender_mac);
    ethernet_frame2.set_source(sender_mac);
    ethernet_frame2.set_destination(MacAddr::broadcast());

    ethernet_frame1.set_ethertype(EtherTypes::Arp);
    ethernet_frame2.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer1 = [0u8; 28];
    let mut arp_buffer2 = [0u8; 28];
    let mut arp_frame1 = MutableArpPacket::new(&mut arp_buffer1).expect("Packet Creation Failed");
    let mut arp_frame2 = MutableArpPacket::new(&mut arp_buffer2).expect("Packet Creation Failed");
    arp_frame1.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_frame2.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_frame1.set_protocol_type(EtherTypes::Ipv4);
    arp_frame2.set_protocol_type(EtherTypes::Ipv4);
    arp_frame1.set_hw_addr_len(6);
    arp_frame2.set_hw_addr_len(6);
    arp_frame1.set_proto_addr_len(4);
    arp_frame2.set_proto_addr_len(4);

    arp_frame1.set_sender_hw_addr(sender_mac);
    arp_frame1.set_sender_proto_addr(arplog.target_ip);
    arp_frame1.set_target_hw_addr(arplog.sender_mac);
    arp_frame1.set_target_proto_addr(arplog.sender_ip);

    arp_frame2.set_sender_hw_addr(sender_mac);
    arp_frame2.set_sender_proto_addr(arplog.sender_ip);
    arp_frame2.set_target_hw_addr(MacAddr::zero());
    arp_frame2.set_target_proto_addr(arplog.target_ip);

    ethernet_frame1.set_payload(arp_frame1.packet());
    ethernet_frame2.set_payload(arp_frame2.packet());

    (ethernet_buffer1, ethernet_buffer2)
}

 */
