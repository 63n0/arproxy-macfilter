use std::{
    env, fs::File, io::BufReader, path::PathBuf, str::FromStr, sync::Arc, thread, time::Duration,
};

mod config;
mod networks;
mod repositories;
mod web;

use clap::Parser;
use config::Args;
use pnet::util::MacAddr;
use repositories::{allowed_mac::AllowedMacRepository, config::ConfigRepository};
use tracing::{debug, error, info, trace, warn};

#[tokio::main]
async fn main() {
    // logging
    let mut log_level: String = "info".to_string();
    if cfg!(debug_assertions) {
        log_level = env::var("RUST_LOG").unwrap_or("debug".to_string());
    }
    env::set_var("RUST_LOG", log_level);

    tracing_subscriber::fmt::init();

    // arguments and configuration
    let args = config::Args::parse();
    let config: config::Config =
        config::load_config(&args.config_path).expect("Failed to load configuration");
    security_checkup(&config, &args);
    trace!("{:?}", config);

    // repository creation
    let allowedmac_repo = repositories::allowed_mac::AllowedMacRepositoryForMemory::new();
    let arplog_repo = repositories::arplog::ArpLogRepositoryForMemory::new();
    let config_repo = repositories::config::ConfigRepositoryForMemory::new(config);

    if let Some(path) = config_repo.get_config().allowed_mac_list {
        let allowed_macs = load_allowed_macs(&path);
        for m in allowed_macs {
            allowedmac_repo.add(m).expect("[Error] SyncErr");
        }
    }

    // network-related
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
    debug!("Packet listener thread spawned");
    let task1 = packet_sender.send_loop();
    debug!("Packet sender task started");
    thread::sleep(Duration::from_millis(300)); // wait for start packet_sender
    if thread1.is_finished() {
        error!("Something went wrong. Make sure it is running with root privileges.");
        panic!();
    }

    // administration
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
        info!(
            "Administration API listening on http://{}:{}",
            admin_config.listen_address, admin_config.listen_port
        );
        axum::serve(listener, app).await.unwrap();
    }
    task1.await;
    thread1.join().unwrap();
}

/// ConfigとArgsについてのセキュリティチェックを行う
fn security_checkup(config: &config::Config, args: &Args) {
    let admin_config = config.administration.clone();
    if !admin_config.listen_address.is_loopback() && !args.insecure {
        error!("Non-loopback address {:?} is not accepted as 'administration.listen_address'. Administration api DOES NOT REQUIRE LOGIN, consider using ssh port forwarding. If you will ignore the warning and use a non-loopback address, enable the `--insecure` argument.", admin_config.listen_address);
        panic!("Exitting..");
    } else if !admin_config.listen_address.is_loopback() {
        warn!("Administration api DOES NOT REQUIRE LOGIN, consider using ssh port forwarding.");
    }
}

fn load_allowed_macs(path: &PathBuf) -> Vec<MacAddr> {
    let file = File::open(path)
        .expect(format!("Failed to open allowed mac list file {:?}", path).as_str());
    let reader = BufReader::new(file);
    let macs_str: Vec<String> = serde_json::from_reader::<BufReader<File>, Vec<String>>(reader)
        .expect(format!("Failed to open allowed mac list file {:?}", path).as_str());
    let macs = macs_str
        .iter()
        .map(|mac_str| MacAddr::from_str(&mac_str).expect("Failed to parse mac address"))
        .collect::<Vec<MacAddr>>();
    macs
}
