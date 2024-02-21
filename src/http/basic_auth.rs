use std::sync::Arc;
use axum::{
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::{IntoResponse, Html},
};
use crate::{
    models::{Param, User},
    http::AppState,
};




pub async fn bauth<B>(
    State(app_state): State<Arc<AppState>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, Html<String>>{
    let token = req.headers()
        .get(header::AUTHORIZATION)
            .and_then(|auth_header| auth_header.to_str().ok())
            .and_then(|auth_value| auth_value.map(|value| value.to_string()));

}
