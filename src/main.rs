use actix_web::{
    dev::ServiceRequest,
    web,
    App,
    HttpServer,
    Error as ActixError,
    error::ErrorUnauthorized,
};
use actix_web_httpauth::{
    extractors::basic::BasicAuth,
    middleware::HttpAuthentication,
};
use dav_server::actix::*;
use dav_server::{memls::MemLs, localfs::LocalFs, DavConfig, DavHandler};
use std::env::var;
use tracing_subscriber::{
    filter::EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use std::str::FromStr;
use tracing::info;
use crate::models::{Error, Config};

mod models;


pub async fn dav_handler(
    req: DavRequest,
    davhandler: web::Data<DavHandler>,
) -> DavResponse {
    if let Some(prefix) = req.prefix() {
        let config = DavConfig::new().strip_prefix(prefix);
        davhandler.handle_with(config, req.request).await.into()
    } else {
        davhandler.handle(req.request).await.into()
    }
}

async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (ActixError, ServiceRequest)> {
    let config: &Config = req.app_data::<web::Data<Config>>()
        .expect("Config data missing in request handler.");
    if config.check_auth(&credentials){
        Ok(req)
    }else{
        Err((ErrorUnauthorized("User not authorized"), req))
    }
}

#[actix_web::main]
async fn main() -> Result<(), Error> {
    let log_level = var("RUST_LOG").unwrap_or("debug".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Log level: {log_level}");

    let config = models::Config::read().await.unwrap();

    let dir = config.get_directory();

    let dav_server = DavHandler::builder()
        .filesystem(LocalFs::new(dir, false, false, false))
        .locksystem(MemLs::new())
        .build_handler();
    let addr = format!("0.0.0.0:{}", config.get_port());

    tracing::info!("🚀 Server started successfully");
    HttpServer::new(move || {
        let auth = HttpAuthentication::basic(validator);
        App::new()
            .app_data(web::Data::new(config.clone()))
            .wrap(auth)
            .app_data(web::Data::new(dav_server.clone()))
            .service(web::resource("/{tail:.*}").to(dav_handler))
    })
    .bind(addr)?
    .run()
    .await
    .map_err(|e|  e.into())
}
