use tower::Service;
use dav_server::actix::*;
use dav_server::{memls::MemLs, localfs::LocalFs, DavConfig, DavHandler};
use axum::{
    handler::{HandlerWithoutStateExt, Handler},
    extract::{State, Request},
};
use std::sync::Arc;
use crate::models::Config;

pub async fn dav_handler(
    State(davhandler): State<Arc<DavHandler>>,
) -> DavResponse {
    if let Some(prefix) = req.prefix() {
        let config = DavConfig::new().strip_prefix(prefix);
        davhandler.handle_with(config, req.request).await.into()
    } else {
        davhandler.handle(req.request).await.into()
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
