use std::{str::FromStr, sync::Arc};

use crate::repositories::allowed_mac::AllowedMacRepository;
use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Result},
    Extension, Json,
};
use pnet::util::MacAddr;
use serde::de::DeserializeOwned;
use tracing::debug;
use validator::Validate;

use super::schema::{AllowedMacDeleteSchema, AllowedMacPostResponseSchema, AllowedMacPostSchema};

#[derive(Debug)]
pub struct ValidatedJson<T>(T);

#[axum::async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "Json parse error.".to_string()))?;
        value
            .validate()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid schema".to_string()))?;
        Ok(ValidatedJson(value))
    }
}

pub async fn add_allowedmac<M: AllowedMacRepository>(
    Extension(allowedmac_repo): Extension<Arc<M>>,
    ValidatedJson(payload): ValidatedJson<AllowedMacPostSchema>,
) -> Result<impl IntoResponse, StatusCode> {
    debug!("Adding MAC addres: {:?}", payload);
    let addr = MacAddr::from_str(&payload.mac_address).map_err(|_| StatusCode::BAD_REQUEST)?;
    let result = allowedmac_repo.add(addr);
    debug!("Adding MAC addres: {:?}", result);
    let created_addr = result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((
        StatusCode::CREATED,
        Json(AllowedMacPostResponseSchema {
            mac_address: created_addr.to_string(),
        }),
    ))
}

pub async fn all_allowedmac<M: AllowedMacRepository>(
    Extension(allowedmac_repo): Extension<Arc<M>>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = allowedmac_repo.getall();
    let addrs = result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let addrs_str: Vec<String> = addrs.iter().map(|addr| addr.to_string()).collect();
    Ok((StatusCode::OK, Json(addrs_str)))
}

pub async fn delete_allowedmac<M: AllowedMacRepository>(
    Extension(allowedmac_repo): Extension<Arc<M>>,
    ValidatedJson(payload): ValidatedJson<AllowedMacDeleteSchema>,
) -> Result<impl IntoResponse, StatusCode> {
    let addr = MacAddr::from_str(&payload.mac_address).map_err(|_| StatusCode::BAD_REQUEST)?;
    let result = allowedmac_repo.remove(&addr);
    debug!("Deleting allowed mac... {:?}", result);
    result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}
