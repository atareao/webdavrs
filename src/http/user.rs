use axum::{
    response::IntoResponse,
    Json,
    http::StatusCode,
};
use serde_json::json;
use axum_auth::AuthBasic;
use sqlx::SqlitePool;
use crate::models::{
    NewUser,
    User,
    Role,
};



pub async fn create_user(
    auth: AuthBasic,
    pool: &SqlitePool,
    Json(newUser): Json<NewUser>,
) -> impl IntoResponse{
    match User::from_auth(&auth, &pool).await {
        Some(user) =>  if user.is_admin(){
                let role = Role::User.to_string();
                match User::create(&pool, &role, &newUser).await {
                    Ok(new_user) => (StatusCode::OK, Json(json!({
                        "result": "ok",
                        "message": "New user created",
                        "content": new_user
                    }))),
                    Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({
                        "result": "ko",
                        "message": e.to_string(),
                    })))
                }
            }else{
                (StatusCode::UNAUTHORIZED, Json(json!({
                    "result": "ko",
                    "message": "Unauthorized"
                })))
            },
        None => (StatusCode::UNAUTHORIZED, Json(json!({
                    "result": "ko",
                    "message": "Unauthorized"
                }))),
    }
}

pub async fn read_user(auth: BasicAuth, pool: web::Data<SqlitePool>, path: web::Path<String>) -> impl Responder{
    let username = path.into_inner();
    match User::from_auth(&auth, &pool).await {
        Some(user) =>  if user.is_admin(){
                match User::read(&pool, &username).await {
                    Ok(user) => HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .body(serde_json::to_string(&user).unwrap()),
                    Err(_) => HttpResponse::UnprocessableEntity().finish(),
                }
            }else{
                HttpResponse::Unauthorized().finish()
            },
        None => HttpResponse::Unauthorized().finish(),
    }
}

#[get("/v1/user")]
pub async fn read_all_users(auth: BasicAuth, pool: web::Data<SqlitePool>) -> impl Responder{
    match User::from_auth(&auth, &pool).await {
        Some(user) =>  if user.is_admin(){
                match User::read_all(&pool).await {
                    Ok(users) => HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .body(serde_json::to_string(&users).unwrap()),
                    Err(_) => HttpResponse::UnprocessableEntity().finish(),
                }
            }else{
                HttpResponse::Unauthorized().finish()
            },
        None => HttpResponse::Unauthorized().finish(),
    }
}

#[delete("/v1/user")]
pub async fn delete_user(auth: BasicAuth, pool: web::Data<SqlitePool>, username: String) -> impl Responder{
    match User::from_auth(&auth, &pool).await {
        Some(user) =>  if user.is_admin(){
                match User::delete(&pool, &username).await {
                    Ok(_) => HttpResponse::Ok().finish(),
                    Err(_) => HttpResponse::NotFound().finish(),
                }
            }else{
                HttpResponse::Unauthorized().finish()
            },
        None => HttpResponse::Unauthorized().finish(),
    }
}
