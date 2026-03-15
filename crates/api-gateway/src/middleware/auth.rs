use axum::{extract::Request, http::header::AUTHORIZATION, middleware::Next, response::Response};
use shared::auth::decode_token;
use shared::AppError;

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, AppError> {
    if req.method() == axum::http::Method::OPTIONS {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization header format".into()))?;

    let claims = decode_token(token)?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
