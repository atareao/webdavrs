use std::sync::Arc;
use axum::{
    Router,
    routing,
    response::{
        IntoResponse,
        Json
    },
    middleware,
    http::{header::{self, HeaderValue}, StatusCode},
};
use base64::{engine::general_purpose, Engine as _};
use dav_server::{
    fakels::FakeLs,
    localfs::LocalFs,
    DavHandler,
};

use crate::{
    http::{
        AppState,
        jwt_auth::auth,
    },
    models::Response,
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
            routing::get(get_dav_handler())
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
}

fn get_dav_handler() -> DavHandler{
    DavHandler::builder()
        .filesystem(LocalFs::new("/", false, false, false))
        .locksystem(FakeLs::new())
        .build_handler()

}

async fn favicon() -> impl IntoResponse {
    let one_pixel_favicon = "";
    let pixel_favicon= general_purpose::STANDARD.decode(one_pixel_favicon).unwrap();
    ([(header::CONTENT_TYPE, HeaderValue::from_static("image/png"))], pixel_favicon)
}

async fn healthcheck() -> impl IntoResponse{
    (
         StatusCode::OK,
         Json(Response{
            status: true,
            message: "Up and running",
            data: None,
         })
    )
}
