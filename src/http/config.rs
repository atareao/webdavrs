use serde::{Serialize, Deserialize};
use std::sync::Arc;

use axum::{
    extract::State,
    Router,
    routing,
    Json,
    http::StatusCode,
    response::IntoResponse,
    middleware,
};


use crate::{
    http::{
        AppState,
        jwt_auth::auth,
    },
    models::{
        Param,
        Response
    },
};
use tracing::{debug, error};

pub fn router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/config",
            routing::get(get_config)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
        .route("/api/v1/config",
            routing::post(post_config)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
}

pub async fn get_config(
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse{
    let params = Param::get_all(&app_state.pool).await?;
    debug!("{:?}", params);
    (StatusCode::OK,
     Json(Response {
            status: true,
            message: "Up and running",
            data: Some(params),
        }))
}

#[derive(Serialize, Deserialize)]
struct KeyValue{
    key: String,
    value: String
}

async fn post_config(
    State(app_state): State<Arc<AppState>>,
    Json(pairs): Json<Vec<KeyValue>>,
) -> impl IntoResponse{
    let mut response_pairs = Vec::new();
    for pair in pairs{
        match Param::set(&app_state.pool, &pair.key, &pair.value).await{
            Ok(kv) => {
                debug!("{:?}", kv);
                let key = kv.get_key(); 
                let value = kv.get_value();
                response_pairs.push(KeyValue{
                    key: key.to_string(),
                    value: value.to_string(),
                });
            },
            Err(e) => {
                error!("{:?}", e);
                response_pairs.push(KeyValue{
                    key: pair.key,
                    value: pair.value,
                });
            }
        }
    }
    (StatusCode::OK, Json(Response{
        status: true,
        message: "ok",
        data: Some(response_pairs),
    }))
}

