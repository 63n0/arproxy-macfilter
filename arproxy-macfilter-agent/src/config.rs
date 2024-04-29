use std::{fs::File, io::BufReader};

use serde::{Deserialize, Serialize};

/// 設定ファイル/設定情報の構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Network interface name
    pub interface: String,
    pub arp_proxy_config: ArpProxyConfig,
}

/// 設定ファイルの一部・プロキシの挙動について定義する
#[derive(Debug, Serialize, Deserialize)]
pub struct ArpProxyConfig {
    /// 許可されたMACアドレスにも応答する場合 true
    pub proxy_allowed_macs: bool,
    /// ARP Replyの送信間隔 (s)
    pub arp_reply_interval: u32,
    /// ARP Replyを送信し続ける時間 (s)
    pub arp_reply_duration: u32,
}

pub fn load_config(filepath: String) -> Result<Config, anyhow::Error> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}
