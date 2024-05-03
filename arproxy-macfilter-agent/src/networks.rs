use std::net::Ipv4Addr;

use pnet::{
    packet::{
        arp::{ArpHardwareTypes, ArpPacket, MutableArpPacket},
        ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
        Packet,
    },
    util::MacAddr,
};

use crate::repositories::ArpLog;

pub async fn proxy_macfilter() {}

fn handle_ethernet(frame: &EthernetPacket) {
    if (frame.get_ethertype() == EtherTypes::Arp) {
        match ArpPacket::new(&frame.payload()) {
            Some(arpframe) => handle_arp(&arpframe),
            None => (),
        }
    }
}

fn handle_arp(frame: &ArpPacket) {}

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
