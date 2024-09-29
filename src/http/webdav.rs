use actix_web::{
    web,
};
use dav_server::actix::*;
use dav_server::{memls::MemLs, localfs::LocalFs, DavConfig, DavHandler};
use crate::models::Config;
use super::render_directory;
use http::Response;
use tracing::info;

pub async fn dav_handler(
    req: DavRequest,
    davhandler: web::Data<DavHandler>,
    config: web::Data<Config>,
) -> DavResponse {
    if let Some(prefix) = req.prefix() {
        let config = DavConfig::new().strip_prefix(prefix);
        davhandler.handle_with(config, req.request).await.into()
    } else if req.request.method() == "GET" && req.request.uri().to_string().ends_with('/') {
        let maindir = config.get_directory();
        let subdir = req.request.uri().to_string();
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
        davhandler.handle(req.request).await.into()
    }
}

pub fn get_dav_server(dir: &str) -> DavHandler {
    DavHandler::builder()
        .filesystem(LocalFs::new(dir, false, false, false))
        .locksystem(MemLs::new())
        .build_handler()
}
