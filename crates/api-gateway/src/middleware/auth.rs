use axum::{extract::Request, http::header::AUTHORIZATION, middleware::Next, response::Response};
use shared::auth::decode_token;
use shared::AppError;

/// The token in `Authorization: Bearer <token>`, `None` when the header is absent.
fn bearer_token(req: &Request) -> Result<Option<&str>, AppError> {
    let Some(header) = req.headers().get(AUTHORIZATION) else {
        return Ok(None);
    };
    header
        .to_str()
        .ok()
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(Some)
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization header format".into()))
}

/// Requires a valid token and puts its claims in the request. Role checks are done by the
/// handlers (`shared::auth::{AuthUser, Staff, Admin}`).
pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, AppError> {
    if req.method() == axum::http::Method::OPTIONS {
        return Ok(next.run(req).await);
    }

    let token = bearer_token(&req)?
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;
    let claims = decode_token(token)?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// For routes anyone may read but only some may change (the catalog): a request without a
/// token goes through as an anonymous visitor, a request with a token must carry a valid one.
/// Either way it is the handler's extractor that decides: `Staff` on a write answers 401 to
/// the anonymous visitor and 403 to a customer.
pub async fn optional_auth_middleware(mut req: Request, next: Next) -> Result<Response, AppError> {
    if let Some(token) = bearer_token(&req)? {
        let claims = decode_token(token)?;
        req.extensions_mut().insert(claims);
    }

    Ok(next.run(req).await)
}
