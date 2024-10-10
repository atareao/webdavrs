use axum::{
    extract::FromRequestParts,
    middleware::from_extractor,
    routing::{get, post},
    Router,
    http::{header, StatusCode, request::Parts},
};


struct RequireDav;

/*
impl<S> FromRequestParts<S> for RequireDav
where 
    S: Send + Sync,
{
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let method = parts.method;
    }

}
*/
