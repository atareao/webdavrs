mod user;
mod estatic;
mod root;
mod config;
mod webdav;

use std::{sync::Arc, net::{SocketAddr, Ipv4Addr}, fmt::format};
use sqlx::sqlite::SqlitePool;
use axum::{
    Router,
    http::{
        header::{
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE
        },
        HeaderValue,
        Method,
    },
};
use minijinja::{Environment, path_loader};
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
};
use once_cell::sync::Lazy;
use crate::models::{
    Param,
    Error,
};

pub static ENV: Lazy<Environment<'static>> = Lazy::new(|| {
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    env
});

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub async fn serve(pool: &SqlitePool) -> Result<(), Error>{

    let url = Param::get_url(pool).await;
    let port = Param::get_port(pool).await;
    let cors = CorsLayer::new()
        .allow_origin(url.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));

    let app = api_router(
            AppState {
                pool: pool.clone(),
            })
            .layer(TraceLayer::new_for_http())
            .layer(cors);
    let uri = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(uri).await?;
    axum::serve(listener, app).await
}


fn api_router(app_state: AppState) -> Router {
    estatic::router()
        .merge(root::router(Arc::new(app_state.clone())))
        .merge(user::router())
        .merge(config::router(Arc::new(app_state.clone())))
        .with_state(Arc::new(app_state.clone()))
}

