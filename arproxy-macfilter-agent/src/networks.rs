use pnet::packet::ethernet::MutableEthernetPacket;

use crate::repositories::ArpLog;




pub async fn proxy_macfilter(){

}


pub async fn construct_proxyarp_arpframe(arplog: ArpLog) -> (ArpLog, ArpLog){
    let mut ethernet_buffer1 = [0u8; 42];
    let mut ethernet_buffer2 = [0u8; 42];
    let mut frame_to_source = MutableEthernetPacket::new(packet)
}