use actix_web::{
    dev::ServiceRequest,
    web,
    Error as ActixError,
    error::ErrorUnauthorized,
};
use actix_web_httpauth::extractors::basic::BasicAuth;
use crate::models::Config;

pub async fn validator(
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

