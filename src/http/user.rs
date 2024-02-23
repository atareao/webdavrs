use std::sync::Arc;
use std::{
    fmt,
    str::FromStr,
};
use axum::{
    extract::{
        State,
        Query,
    },
    Router,
    routing,
    middleware,
    response::IntoResponse,
    Json,
    http::StatusCode,
};
use serde::{de, Deserialize, Deserializer};
use crate::{
    http::{
        AppState,
        jwt_auth::auth,
    },
    models::{
        Response,
        NewUser,
        User,
        Role,
    }
};

pub fn router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/users",
            routing::post(create_user)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
        .route("/api/v1/users",
            routing::get(read_user)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
        .route("/api/v1/users",
            routing::delete(delete_user)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
}



pub async fn create_user(
    State(app_state): State<Arc<AppState>>,
    Json(newUser): Json<NewUser>,
) -> impl IntoResponse{
    let role = Role::User.to_string();
    match User::create(&app_state.pool, &role, &newUser).await {
        Ok(new_user) => (StatusCode::OK, Json(Response{
            status: true,
            message: "New user created",
            data: Some(new_user)
        })),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, Json(Response{
            status: false,
            message: &e.to_string(),
            data: None
        }))
    }
}

#[derive(Debug, Deserialize)]
struct Params {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    username: Option<String>,
}

pub async fn read_user(
    State(app_state): State<Arc<AppState>>,
    Query(Params{username}): Query<Params>,
) -> impl IntoResponse{
    match username {
        Some(username) => read_user(app_state, username).await, 
        None => read_all_users(app_state).await,
    }
}

pub async fn read_one_user(
    app_state: Arc<AppState>,
    username: String,
) -> impl IntoResponse{
    match User::read(&app_state, &username).await {
        Ok(user) => (StatusCode::OK, Json(Response{
            status: true,
            message: "User found",
            data: Some(user)
        })),
        Err(e) => (StatusCode::NOT_FOUND, Json(Response{
            status: false,
            message: &e.to_string(),
            data: None
        }))
    }
}

pub async fn read_all_users(
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match User::read_all(&app_state.pool).await {
        Ok(users) => (StatusCode::OK, Json(Response{
            status: true,
            message: "All users",
            data: Some(users)
        })),
        Err(e) => (StatusCode::NOT_FOUND, Json(Response{
            status: false,
            message: &e.to_string(),
            data: None
        }))
    }
}

pub async fn delete_user(
    State(app_state): State<Arc<AppState>>,
    Query(username): Query<String>,
) -> impl IntoResponse{
    match User::delete(&app_state.pool, &username).await {
        Ok(user) => (StatusCode::OK, Json(Response{
            status: true,
            message: "Delete user",
            data: Some(user)
        })),
        Err(e) => (StatusCode::NOT_FOUND, Json(Response{
            status: false,
            message: &e.to_string(),
            data: None
        }))
    }
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}
