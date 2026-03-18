use crate::dto::auth::{LoginRequest, LoginResponse, RegisterRequest};
use crate::usecase::{
    auth::AuthUsecase, change_password::ChangePasswordUsecase, delete_user::DeleteUserUsecase,
    get_me::GetMeUsecase, get_membership_summary::GetMembershipSummaryUsecase,
    list_memberships::ListMembershipsUsecase, list_users::ListUsersUsecase,
    update_profile::UpdateProfileUsecase,
    create_staff::CreateStaffUsecase, list_staff::ListStaffUsecase,
};
use axum::{extract::State, routing::post, Json, Router};
use shared::AppError;
use std::sync::Arc;

pub trait IdentityState: Send + Sync {
    fn auth_service(&self) -> Arc<AuthUsecase>;
    fn get_me_usecase(&self) -> Arc<GetMeUsecase>;
    fn update_profile_usecase(&self) -> Arc<UpdateProfileUsecase>;
    fn list_users_usecase(&self) -> Arc<ListUsersUsecase>;
    fn list_memberships_usecase(&self) -> Arc<ListMembershipsUsecase>;
    fn get_membership_summary_usecase(&self) -> Arc<GetMembershipSummaryUsecase>;
    fn change_password_usecase(&self) -> Arc<ChangePasswordUsecase>;
    fn delete_user_usecase(&self) -> Arc<DeleteUserUsecase>;
    fn list_staff_usecase(&self) -> Arc<ListStaffUsecase>;
    fn create_staff_usecase(&self) -> Arc<CreateStaffUsecase>;
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
        .route("/me/password", post(change_password_handler::<S>))
        .route("/users", axum::routing::get(list_users_handler::<S>))
        .route(
            "/users/{id}",
            axum::routing::delete(delete_user_handler::<S>),
        )
        .route(
            "/memberships",
            axum::routing::get(list_memberships_handler::<S>),
        )
        .route(
            "/memberships/summary",
            axum::routing::get(get_membership_summary_handler::<S>),
        )
        .route("/staff", axum::routing::get(list_staff_handler::<S>).post(create_staff_handler::<S>))
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

async fn list_memberships_handler<S>(
    State(state): State<S>,
) -> Result<Json<Vec<crate::domain::model::UserMembership>>, AppError>
where
    S: IdentityState,
{
    let usecase = state.list_memberships_usecase();
    let members = usecase.execute().await?;
    Ok(Json(members))
}

async fn get_membership_summary_handler<S>(
    State(state): State<S>,
) -> Result<Json<crate::usecase::get_membership_summary::MembershipSummary>, AppError>
where
    S: IdentityState,
{
    let usecase = state.get_membership_summary_usecase();
    let summary = usecase.execute().await?;
    Ok(Json(summary))
}

#[derive(Debug, serde::Deserialize)]
pub struct ChangePasswordRequest {
    pub new_password: String,
}

async fn change_password_handler<S>(
    State(state): State<S>,
    axum::extract::Extension(claims): axum::extract::Extension<shared::auth::UserClaims>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError>
where
    S: IdentityState,
{
    let usecase = state.change_password_usecase();
    usecase.execute(claims.sub, payload.new_password).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn delete_user_handler<S>(
    State(state): State<S>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError>
where
    S: IdentityState,
{
    let usecase = state.delete_user_usecase();
    usecase.execute(id).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn list_staff_handler<S>(State(state): State<S>) -> Result<Json<Vec<crate::dto::staff::StaffMember>>, AppError>
where
    S: IdentityState,
{
    let usecase = state.list_staff_usecase();
    let staff = usecase.execute().await?;
    Ok(Json(staff))
}

#[derive(serde::Deserialize)]
pub struct CreateStaffRequest {
    pub email: String,
    pub password: String,
}

async fn create_staff_handler<S>(
    State(state): State<S>,
    Json(payload): Json<CreateStaffRequest>,
) -> Result<Json<crate::dto::staff::StaffMember>, AppError>
where
    S: IdentityState,
{
    let usecase = state.create_staff_usecase();
    let user = usecase.execute(payload.email, payload.password).await?;
    Ok(Json(user))
}
