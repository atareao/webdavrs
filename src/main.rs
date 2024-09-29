mod models;
mod http;

use actix_web::{
    web,
    App,
    HttpServer,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use std::env::var;
use tracing_subscriber::{
    filter::EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use std::str::FromStr;
use tracing::info;

use models::Error;
use http::{
    dav_handler,
    get_dav_server,
    validator,
};

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
    let workers = config.get_workers();
    let addr = format!("0.0.0.0:{}", config.get_port());

    tracing::info!("🚀 Server started successfully");
    HttpServer::new(move || {
        let auth = HttpAuthentication::basic(validator);
        App::new()
            .app_data(web::Data::new(config.clone()))
            .wrap(auth)
            .app_data(web::Data::new(get_dav_server(&dir)))
            //.service(index)
            .service(web::resource("/{tail:.*}").to(dav_handler))
    })
    .bind(addr)?
    .workers(workers)
    .run()
    .await
    .map_err(|e|  e.into())
}
