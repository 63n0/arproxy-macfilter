use std::sync::Arc;

use crate::repositories::{
    allowed_mac::AllowedMacRepository, arplog::ArpLogRepository, config::ConfigRepository,
};
use axum::Router;

use super::api;

pub fn create_router<C, M, A>(
    config_repo: Arc<C>,
    allowedmac_repo: Arc<M>,
    arplog_repo: Arc<A>,
) -> Router
where
    C: ConfigRepository,
    M: AllowedMacRepository,
    A: ArpLogRepository,
{
    let app = Router::new().nest(
        "/api",
        api::route::create_router(config_repo, allowedmac_repo, arplog_repo),
    );
    app
}
