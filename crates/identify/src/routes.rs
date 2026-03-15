use crate::dto::auth::{LoginRequest, LoginResponse, RegisterRequest};
use crate::usecase::{
    auth::AuthUsecase, get_me::GetMeUsecase, list_users::ListUsersUsecase,
    update_profile::UpdateProfileUsecase,
};
use axum::{extract::State, routing::post, Json, Router};
use shared::AppError;
use std::sync::Arc;

pub trait IdentityState: Send + Sync {
    fn auth_service(&self) -> Arc<AuthUsecase>;
    fn get_me_usecase(&self) -> Arc<GetMeUsecase>;
    fn update_profile_usecase(&self) -> Arc<UpdateProfileUsecase>;
    fn list_users_usecase(&self) -> Arc<ListUsersUsecase>;
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateProfileRequest {
    pub email: String,
}

pub fn init<S>() -> Router<S>
where
    S: IdentityState + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/register", post(register_handler::<S>))
        .route("/login", post(login_handler::<S>))
        .route(
            "/me",
            axum::routing::get(get_me_handler::<S>).patch(update_profile_handler::<S>),
        )
        .route("/users", axum::routing::get(list_users_handler::<S>))
}

async fn register_handler<S>(
    State(state): State<S>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, AppError>
where
    S: IdentityState,
{
    if payload.password != payload.password_confirm {
        return Err(AppError::Validation("Passwords do not match".into()));
    }
    let auth_svc = state.auth_service();
    auth_svc.register(payload.email, payload.password).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn login_handler<S>(
    State(state): State<S>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError>
where
    S: IdentityState,
{
    let auth_svc = state.auth_service();
    let token = auth_svc.login(&payload.email, &payload.password).await?;
    Ok(Json(LoginResponse { token }))
}

async fn get_me_handler<S>(
    State(state): State<S>,
    axum::extract::Extension(claims): axum::extract::Extension<shared::auth::UserClaims>,
) -> Result<Json<crate::domain::model::User>, AppError>
where
    S: IdentityState,
{
    let usecase = state.get_me_usecase();
    let user = usecase.execute(claims.sub).await?;
    Ok(Json(user))
}

async fn update_profile_handler<S>(
    State(state): State<S>,
    axum::extract::Extension(claims): axum::extract::Extension<shared::auth::UserClaims>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<serde_json::Value>, AppError>
where
    S: IdentityState,
{
    let usecase = state.update_profile_usecase();
    usecase.execute(claims.sub, payload.email).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn list_users_handler<S>(
    State(state): State<S>,
) -> Result<Json<Vec<crate::domain::model::User>>, AppError>
where
    S: IdentityState,
{
    let usecase = state.list_users_usecase();
    let users = usecase.execute().await?;
    Ok(Json(users))
}
