use std::{str::FromStr, sync::Arc};

use arproxy_macfilter_agent::repositories::allowed_mac::AllowedMacRepository;
use axum::{http::StatusCode, response::{IntoResponse, Result}, Extension, Json};
use pnet::util::MacAddr;
use tracing::debug;

use super::schema::{AllowedMacDeleteSchema, AllowedMacPostResponseSchema, AllowedMacPostSchema};


pub async fn add_allowedmac<M: AllowedMacRepository>(
        Extension(allowedmac_repo): Extension<Arc<M>>,
        Json(payload): Json<AllowedMacPostSchema>,
    ) -> Result<impl IntoResponse, StatusCode> {
        debug!("Adding MAC addres: {:?}", payload);
        let addr = MacAddr::from_str(&payload.mac_address).unwrap();
        let result = allowedmac_repo.add(addr);
        debug!("Adding MAC addres: {:?}", result);
        let created_addr = result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok((StatusCode::CREATED, Json(AllowedMacPostResponseSchema { mac_address: created_addr.to_string() })))
}

pub async fn all_allowedmac<M: AllowedMacRepository>(
    Extension(allowedmac_repo): Extension<Arc<M>>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = allowedmac_repo.getall();
    let addrs = result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let addrs_str:Vec<String> = addrs.iter().map(|addr| addr.to_string()).collect();
    Ok((StatusCode::OK, Json(addrs_str)))
}

pub async fn delete_allowedmac<M: AllowedMacRepository>(
    Extension(allowedmac_repo): Extension<Arc<M>>,
    Json(payload): Json<AllowedMacDeleteSchema>,
) -> Result<impl IntoResponse, StatusCode> {
    let addr = MacAddr::from_str(&payload.mac_address).unwrap();
    let result = allowedmac_repo.remove(&addr);
    debug!("Deleting allowed mac... {:?}", result);
    result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}