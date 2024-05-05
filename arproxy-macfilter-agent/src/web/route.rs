use std::sync::Arc;

use arproxy_macfilter_agent::repositories::{config::ConfigRepository,  allowed_mac::AllowedMacRepository, arplog::ArpLogRepository};
use axum::Router;

use super::api;



pub fn create_router<C, M, A>(
    config_repo:Arc<C>, 
    allowedmac_repo: Arc<M>, 
    arplog_repo: Arc<A>,
) -> Router 
where
    C: ConfigRepository,
    M: AllowedMacRepository,
    A: ArpLogRepository,
{
    let app = Router::new()
        .nest("/api", api::route::create_router(config_repo, allowedmac_repo, arplog_repo));
    app
}

