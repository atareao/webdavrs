use tower::Service;
use dav_server::actix::*;
use dav_server::{memls::MemLs, localfs::LocalFs, DavConfig, DavHandler};
use axum::{
    handler::{HandlerWithoutStateExt, Handler},
    extract::{State, Request},
};
use tracing::info;
use std::sync::Arc;
use crate::models::Config;

pub async fn dav_handler(
    request: Request,
    State(config): State<Config>,
    State(davhandler): State<Arc<DavHandler>>,
) -> DavResponse {
    let prefix = request.uri().path();
    if request.method() == "GET" && request.uri().to_string().ends_with('/') {
        let maindir = config.get_directory();
        let subdir = request.uri().to_string();
        info!("maindir: {}", &maindir);
        info!("subdir: {}", &subdir);
        if let Some(content) = render_directory(&maindir, &subdir).await{
            let body: dav_server::body::Body = content.into();
            let res = Response::builder()
                .body(body).unwrap();
                DavResponse(res)
        } else {
            DavResponse(Response::builder().status(404).body(dav_server::body::Body::empty()).unwrap())
        }
    } else {
        davhandler.handle(request).await.into()
    }
}

//assert_service(dav_handler);


pub fn get_dav_server(dir: &str) -> DavHandler {
    DavHandler::builder()
        .filesystem(LocalFs::new(dir, false, false, false))
        .locksystem(MemLs::new())
        .build_handler()
}

fn assert_service<S>(service: S)
where
    S: Service<Request>
{}
