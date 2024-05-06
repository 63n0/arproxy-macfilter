use std::{fs::File, io::BufReader, net::Ipv4Addr};

use serde::{Deserialize, Serialize};


/// 設定ファイル/設定情報の構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Network interface name
    pub interface: String,
    pub arp_proxy: ArpProxyConfig,
    pub administration: AdministrationConfig,
}

/// 設定ファイルの一部・プロキシの挙動について定義する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArpProxyConfig {
    /// 許可されたMACアドレスにも応答する場合 true
    pub proxy_allowed_macs: bool,
    /// ARP Replyの送信間隔 (s)
    pub arp_reply_interval: u32,
    /// ARP Replyを送信し続ける時間 (s)
    pub arp_reply_duration: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdministrationConfig {
    /// 管理用APIを有効化
    pub enable_api: bool,
    /// 管理用API/管理画面にバインドするIPアドレス
    pub listen_address: Ipv4Addr,
    /// 管理用API/管理画面にバインドするポート番号
    pub listen_port: u16,
}

pub fn load_config(filepath: String) -> Result<Config, anyhow::Error> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}
