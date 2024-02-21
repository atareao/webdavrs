use std::sync::Arc;
use axum::{
    Router,
    routing,
    middleware,
};
use dav_server::{
    fakels::FakeLs,
    localfs::LocalFs,
    DavHandler,
};

use crate::{
    http::{
        AppState,
    },
};


pub fn router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/favicon.ico",
            routing::get(favicon)
        )
        .route("/healthcheck",
            routing::get(healthcheck)
        )
        .route("/",
            routing::get(get_dav_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
}

fn get_dav_handler() -> DavHandler{
    DavHandler::builder()
        .filesystem(LocalFs::new("/", false, false, false))
        .locksystem(FakeLs::new())
        .build_handler()

}
