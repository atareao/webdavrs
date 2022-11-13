mod list;

use actix_web::{App, web::{self, Data}, HttpServer, middleware::Logger};
use dotenv::dotenv;
use sqlx::SqlitePool;
use std::time;
use std::env;
use env_logger::Env;
use log::info;
use dav_server::{fakels::FakeLs, localfs::LocalFs, DavHandler, actix::{DavRequest, DavResponse}};
use actix_web_httpauth::extractors::basic::{BasicAuth, self};
use crate::list::Lister;


pub async fn dav_handler(auth: BasicAuth, req: DavRequest, davhandler: Data<DavHandler>, folder: Data<String>) -> DavResponse{
    info!("{}", auth.user_id());
    if req.prefix().is_some(){
        let res = http::Response::builder()
            .body(dav_server::body::Body::empty())
            .unwrap();
        DavResponse(res)
    }else{
        info!("{:?}", "aqui");
        info!("{:?}", req.prefix());
        info!("{:?}", req.request.method());
        if req.request.method() == "GET" && req.request.uri().to_string().ends_with("/"){
            let maindir = folder.into_inner().to_string();
            let subdir = req.request.uri().to_string();
            info!("maindir: {}", &maindir);
            info!("subdir: {}", &subdir);
            let lister = Lister::new("Ejemplo", &maindir, &subdir);
            println!("{}", lister.generate().await);
            let content = lister.generate().await;
            let body: dav_server::body::Body = content.into();
            let res = http::Response::builder()
                .body(body).unwrap();
                DavResponse(res)
        }else{
            davhandler.handle(req.request).await.into()
        }
    }

}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port = env::var("PORT").expect("PORT not set");
    let folder = std::env::var("FOLDER").expect("FOLDER not set");


    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let dav_server = DavHandler::builder()
        .filesystem(LocalFs::new(&folder, false, false, false))
        .locksystem(FakeLs::new())
        .build_handler();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(basic::Config::default().realm("Restricted area"))
            .app_data(Data::new(dav_server.clone()))
            .app_data(Data::new(folder.clone()))
            .service(web::resource("/{tail:.*}").to(dav_handler))
    })
    .workers(2)
    .bind(format!("0.0.0.0:{}", &port))
    .unwrap()
    .run()
    .await
}
