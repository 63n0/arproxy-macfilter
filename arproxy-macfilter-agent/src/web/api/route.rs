use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Extension, Router,
};

use crate::repositories::{
    allowed_mac::AllowedMacRepository, arplog::ArpLogRepository, config::ConfigRepository,
};

use super::handlers;

pub fn create_router<C, M, A>(
    _config_repo: Arc<C>,
    allowedmac_repo: Arc<M>,
    _arplog_repo: Arc<A>,
) -> Router
where
    C: ConfigRepository,
    M: AllowedMacRepository,
    A: ArpLogRepository,
{
    let app = Router::new().nest("/allowed-mac", create_allowedmac_router(allowedmac_repo));
    app
}

fn create_allowedmac_router<M>(allowedmac_repo: Arc<M>) -> Router
where
    M: AllowedMacRepository,
{
    let app = Router::new()
        .route("/all", get(handlers::all_allowedmac::<M>))
        .route("/add", post(handlers::add_allowedmac::<M>))
        .route("/delete", delete(handlers::delete_allowedmac::<M>))
        .layer(Extension(allowedmac_repo.clone()));
    app
}

#[cfg(test)]
mod test {
    use std::{str::FromStr, sync::Arc};

    use crate::repositories::allowed_mac::{AllowedMacRepository, AllowedMacRepositoryForMemory};
    use axum::{
        body::{Body, Bytes},
        http::{self, Method, Request, StatusCode},
        Router,
    };
    use http_body_util::BodyExt;
    use pnet::util::MacAddr;
    use tower::ServiceExt;
    use tracing::trace;
    use validator::Validate;

    use crate::web::api::schema::{
        AllowedMacDeleteSchema, AllowedMacPostResponseSchema, AllowedMacPostSchema,
    };

    use super::create_allowedmac_router;

    fn create_dummy_allowedmac_repo() -> AllowedMacRepositoryForMemory {
        let repo = AllowedMacRepositoryForMemory::new();
        repo.add(MacAddr::new(2, 0, 0, 0, 0xf, 1)).expect("SyncErr");
        repo.add(MacAddr::new(2, 0, 0, 0, 0xf, 2)).expect("SyncErr");
        repo.add(MacAddr::new(2, 0, 0, 0, 0xf, 3)).expect("SyncErr");
        repo
    }

    async fn request_oneshot_empty(
        app: Router,
        method: http::Method,
        path: &str,
    ) -> (StatusCode, Bytes) {
        let req = Request::builder()
            .method(method)
            .uri(path)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().collect().await.unwrap().to_bytes();
        trace!("{:?}", body);
        (status, body)
    }

    async fn request_oneshot_json(
        app: Router,
        method: http::Method,
        path: &str,
        body: Vec<u8>,
    ) -> (StatusCode, Bytes) {
        trace!("Request body: {:?}", std::str::from_utf8(&body));
        let req = Request::builder()
            .method(method)
            .uri(path)
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(body))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        trace!("{:?}", res);
        let status = res.status();
        let body = res.into_body().collect().await.unwrap().to_bytes();
        trace!("{:?}", body);
        (status, body)
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn should_getall_allowedmac() {
        let repo = create_dummy_allowedmac_repo();
        let app = create_allowedmac_router(Arc::new(repo));
        // ステータスコード・レスポンスボディが正当か
        let (status, body) = request_oneshot_empty(app, http::Method::GET, "/all").await;
        assert_eq!(status, StatusCode::OK);
        let maddrs_str = serde_json::from_slice::<Vec<String>>(&body).unwrap();
        let maddrs: Vec<MacAddr> = maddrs_str
            .iter()
            .map(|m| MacAddr::from_str(m).unwrap())
            .collect();
        assert_eq!(maddrs.len(), 3);
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 1)));
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 2)));
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 3)));
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn should_add_allowedmac() {
        let repo = create_dummy_allowedmac_repo();
        let app = create_allowedmac_router(Arc::new(repo.clone()));
        let req_body = AllowedMacPostSchema {
            mac_address: MacAddr::new(2, 0, 0, 0, 0xf, 5).to_string(),
        };
        let req_body_raw = serde_json::to_vec(&req_body).unwrap();
        // ステータスコード・レスポンスボディが正当か
        let (status, body) =
            request_oneshot_json(app, http::Method::POST, "/add", req_body_raw).await;
        assert_eq!(status, StatusCode::CREATED);
        let res_body = serde_json::from_slice::<AllowedMacPostResponseSchema>(&body).unwrap();
        res_body.validate().expect("Unexpected response");
        assert_eq!(res_body, req_body);
        // レポジトリに追加済みか
        let maddrs = repo.getall().unwrap();
        assert_eq!(maddrs.len(), 4);
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 1)));
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 2)));
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 3)));
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 5)));
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn should_delete_allowedmac() {
        let repo = create_dummy_allowedmac_repo();
        let app = create_allowedmac_router(Arc::new(repo.clone()));
        let req_body = AllowedMacDeleteSchema {
            mac_address: MacAddr::new(2, 0, 0, 0, 0xf, 2).to_string(),
        };
        let req_body = serde_json::to_vec(&req_body).unwrap();
        // ステータスコードが正当か
        let (status, _) = request_oneshot_json(app, Method::DELETE, "/delete", req_body).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        // レポジトリから削除されているか
        let maddrs = repo.getall().unwrap();
        assert_eq!(maddrs.len(), 2);
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 1)));
        assert!(!maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 2)));
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 3)));
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn invalid_input_allowedmac() {
        let repo = create_dummy_allowedmac_repo();
        let app = create_allowedmac_router(Arc::new(repo.clone()));
        // 無効な入力：非JSON, 不正なMACアドレス
        let req_bodys = vec![
            "{ maddr: true }".to_string().into_bytes(),
            r#"{ "mac_address": "hello, world" }"#.to_string().into_bytes(),
        ];
        for req_body in req_bodys.iter() {
            // ステータスコードが正当か
            let (status, _) =
                request_oneshot_json(app.clone(), Method::DELETE, "/delete", req_body.clone())
                    .await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
        }
        for req_body in req_bodys {
            // ステータスコードが正当か
            let (status, _) =
                request_oneshot_json(app.clone(), Method::POST, "/add", req_body.clone()).await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
        }

        // レポジトリに変化がないか
        let maddrs = repo.getall().unwrap();
        assert_eq!(maddrs.len(), 3);
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 1)));
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 2)));
        assert!(maddrs.contains(&MacAddr::new(2, 0, 0, 0, 0xf, 3)));
    }
}
