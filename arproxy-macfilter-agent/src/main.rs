use std::{sync::Arc, thread};

mod config;
mod networks;
mod repositories;
mod web;

use repositories::config::ConfigRepository;
use tracing::trace;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config_str = r#"{
        "interface":"lo",
        "arp_proxy": {
            "proxy_allowed_macs": false,
            "arp_reply_interval": 1,
            "arp_reply_duration": 10
        }, 
        "administration": {
            "enable_api": true,
            "listen_address": "127.0.0.1",
            "listen_port": 3000
        }
    }
    "#;
    let config: config::Config = serde_json::from_str(&config_str).unwrap();
    trace!("{:?}", config);
    let allowedmac_repo = repositories::allowed_mac::AllowedMacRepositoryForMemory::new();
    let arplog_repo = repositories::arplog::ArpLogRepositoryForMemory::new();
    let config_repo = repositories::config::ConfigRepositoryForMemory::new(config);

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

    let thread1 = thread::spawn(move || {
        packet_listener.listen();
    });
    let task1 = packet_sender.send_loop();

    let admin_config = config_repo.get_config().administration;
    if admin_config.enable_api {
        let app = web::route::create_router(
            Arc::new(config_repo.clone()),
            Arc::new(allowedmac_repo.clone()),
            Arc::new(arplog_repo.clone()),
        );
        let listener =
            tokio::net::TcpListener::bind((admin_config.listen_address, admin_config.listen_port))
                .await
                .expect(
                    format!(
                        "Failed to bind TCP listener to address {} and port {}",
                        admin_config.listen_address, admin_config.listen_port
                    )
                    .as_str(),
                );
        axum::serve(listener, app).await.unwrap();
    }
    task1.await;
    thread1.join().unwrap();
}
