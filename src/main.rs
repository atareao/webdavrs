mod models;
mod http;

use std::env::var;
use axum::{
    extract::State,
    routing::{get, Router},
    middleware::from_fn_with_state,
};
use axum::response::IntoResponse;
use tracing_subscriber::{
    filter::EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use std::str::FromStr;
use tracing::info;
use crate::http::index;

use models::Error;
use http::auth_middleware;
use dav_server::{fakels::FakeLs, localfs::LocalFs, DavHandler};


pub async fn dav_handler(State(dav_server): State<DavHandler>) -> impl IntoResponse {
    //Ok::<_, Infallible>(dav_server.handle(req).await)
    //let response = dav_server.handle(req).await;
    //response.into_response()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let log_level = var("RUST_LOG").unwrap_or("debug".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Log level: {log_level}");

    let config = models::Config::read().await?;
    let addr = format!("0.0.0.0:{}", config.get_port());

    let dav_server = DavHandler::builder()
        .filesystem(LocalFs::new(config.get_directory(),
            false, false, false))
        .locksystem(FakeLs::new())
        .build_handler();

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚀 Server started successfully");
    let app = Router::new()
        //.route("/", get(index))
        .route("/*tail", get(index))
        //.route("/*tail", get(index))
        //.route("/*tail", any(dav_handler))
        /*
        .route_service(
            "tail", 
            service_fn(|req: Request| async move {
                info!("Request received");
                //let dav_handler = req.extensions().get::<State<DavHandler>>().unwrap();
                if let Some(State(dav_server)) =  req.extensions().get::<State<DavHandler>>(){
                    Ok::<_, Infallible>(dav_server.handle(req).await)
                    //Ok::<_, Infallible>(Response::builder().status(404).body(dav_server::body::Body::empty()).unwrap())
                } else {
                    
                    Ok::<_, Infallible>(Response::builder().status(404).body(dav_server::body::Body::empty()).unwrap())

                    //Ok::<_, Infallible>(dav_server::body::Body::empty())
                }
                //info!("Request received");

                //let dav_server = dav_server.clone();
                //Ok::<_, Infallible>(dav_server.handle(req).await)
            })
        )
        */
        //.with_state(dav_server)
        .with_state(config.clone())
        .route_layer(from_fn_with_state(config.clone(), auth_middleware));
    axum::serve(listener, app).await
        .map_err(|e|  e.into())
}

