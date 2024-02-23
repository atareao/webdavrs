use std::sync::Arc;
use axum::{
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::{IntoResponse, Html},
};

use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Serialize;
use minijinja::context;

use super::AppState;
use super::super::models::Param;
use super::ENV;

use crate::models::{
    User,
    TokenClaims
};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: &'static str,
}

pub async fn auth<B>(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
    mut req: Request<B>,
    next: Next,
) -> Result<impl IntoResponse, Html<String>> {
    let token = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    auth_value.strip_prefix("Bearer ")
                        .map(|value| value.to_string())
                })
        });

    let token = token.ok_or_else(|| {
        let msg = "You are not logged. Please <a href='/login'>log in</a>";
        get_html_error(&app_state, msg)
    })?;

    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(
            Param::get_secret(&app_state.pool).await.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| {
        let msg = "Invalid token. Please <a href='/login'>log in</a>";
        get_html_error(&app_state, msg)
    })?
    .claims;

    let user_name = &claims.sub.to_string();
    let user = User::get_by_name(&app_state.pool, user_name)
        .await
        .map_err(|_e| {
        let msg = "The user belonging to this token no longer exists. Please <a href='/login'>log in</a>";
        get_html_error(&app_state, msg)
    })?;


    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

fn get_html_error(_app_state: &Arc<AppState>, msg: &str) -> Html<String>{
    let template = ENV.get_template("error.html").unwrap();
    let ctx = context! {
        title             => "Podmixer",
        error_title       => "Error",
        error_description => msg,
    };
    Html(template.render(ctx).unwrap())
}


