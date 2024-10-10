use axum::{
    body::Body,
    extract::{State, Request},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::prelude::*;
use tracing::debug;
use super::super::models::Config;


pub async fn auth_middleware(
    State(config): State<Config>,
    request: Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    match auth_header {
        Some(auth_header) if token_is_valid(auth_header, &config) => {
            debug!("Authorized");
            next.run(request).await
        }
        _ => {
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap()
        }
    }

}

fn token_is_valid(auth_header: &str, config: &Config) -> bool {
    if auth_header.starts_with("Basic ") {
        let token = auth_header.trim_start_matches("Basic ");
        BASE64_STANDARD.decode(token)
            .map_err(|_| "Invalid base64")
            .and_then(|decoded| String::from_utf8(decoded).map_err(|_| "Invalid utf8"))
            .and_then(|decoded| {
                debug!(decoded);
                let auth_basic = decoded.split(':').collect::<Vec<&str>>();
                if config.check_auth(auth_basic) {
                    Ok(())
                } else {
                    Err("Ni idea")
                }
            }).is_ok()
    }else{
        false
    }
}
