use actix_web::web;
use dav_server::actix::*;
use dav_server::{memls::MemLs, localfs::LocalFs, DavConfig, DavHandler};

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

pub fn get_dav_server(dir: &str) -> DavHandler {
    DavHandler::builder()
        .filesystem(LocalFs::new(dir, false, false, false))
        .locksystem(MemLs::new())
        .build_handler()
}
