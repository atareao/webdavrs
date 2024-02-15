use serde::{Serialize, Deserialize};
use std::sync::Arc;
use serde_json::json;

use axum::{
    extract::State,
    Router,
    routing,
    response::{
        Html,
        IntoResponse,
    },
    Json,
    middleware::from_fn_with_state
};
use minijinja::{
    context,
    value::Value,
};


use crate::{
    models::Param,
    http::{
        AppState,
        jwt_auth::auth
    },
};
use tracing::{debug, error};
use super::ENV;

pub fn router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/config",
            routing::get(get_config)
            .route_layer(from_fn_with_state(app_state.clone(), auth))
        )
        .route("/config",
            routing::post(post_config)
            .route_layer(from_fn_with_state(app_state.clone(), auth))
        )
}

pub async fn get_config(
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse{
    let params = Param::get_all(&app_state.pool).await.unwrap();
    debug!("{:?}", params);
    let template = ENV.get_template("config.html").unwrap();
    let ctx = context!{
        title => "PodMixer",
        ..Value::from_serializable(&params),
    };
    //let ctx = context! {
    //    title => "PodMixer",
    //    feed_title => params.get("feed.title"),
    //    feed_link => params.get("feed.link"),
    //    feed_image_url => params.get("feed.image_url"),
    //    feed_category => params.get("feed.category"),
    //    feed_rating => params.get("feed.rating"),
    //    telegram_token => params.get("telegram.token"),
    //    telegram_chat_id => params.get("telegram.chat_id"),
    //    telegram_thread_id => params.get("telegram.thread_id"),
    //};
    Html(template.render(ctx).unwrap())
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
    Json(json!({
        "result": "ok",
        "content": response_pairs,
    }))
}

