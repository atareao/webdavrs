mod models;
mod http;

use std::env::var;
use std::convert::Infallible;
use axum::{
    extract::{State, Request},
    routing::{get, Router},
};
use tower::service_fn;
use tracing_subscriber::{
    filter::EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use std::str::FromStr;
use tracing::info;
use std::sync::Arc;

use models::Error;
use http::{
    dav_handler,
    get_dav_server,
};
use dav_server::{fakels::FakeLs, memls::MemLs, localfs::LocalFs, DavConfig, DavHandler};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let log_level = var("RUST_LOG").unwrap_or("debug".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Log level: {log_level}");

    let config = models::Config::read().await?;
    let dir = config.get_directory();
    let addr = format!("0.0.0.0:{}", config.get_port());

    let dav_server = DavHandler::builder()
        .filesystem(LocalFs::new(dir, false, false, false))
        .locksystem(FakeLs::new())
        .build_handler();

    tracing::info!("🚀 Server started successfully");
    let app = Router::new()
        .route("/", get(index))
        .route("/*tail", get(index))
        .route_service("/dav", dav_handler)
        .route_service("/dav", service_fn(|req: Request| async move {
            let dav_server = dav_server.clone();
            Ok::<_, Infallible>(dav_server.handle(req).await)
        }))
        .with_state(config)
        .with_state(Arc::new(get_dav_server(&dir)));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await
        .map_err(|e|  e.into())
}
