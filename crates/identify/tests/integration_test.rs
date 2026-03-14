use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use identify::{
    domain::user_repository::UserRepository,
    infrastructure::persistence::postgres::PostgresUserRepository,
    usecase::{
        auth::AuthUsecase,
        get_me::GetMeUsecase,
        update_profile::UpdateProfileUsecase,
        list_users::ListUsersUsecase,
    },
};

struct TestContext {
    auth: AuthUsecase,
    get_me: GetMeUsecase,
    update_profile: UpdateProfileUsecase,
    list_users: ListUsersUsecase,
    repo: Arc<PostgresUserRepository>,
}

impl TestContext {
    fn new(pool: PgPool) -> Self {
        let repo = Arc::new(PostgresUserRepository::new(Arc::new(pool.clone())));
        Self {
            auth: AuthUsecase::new(repo.clone()),
            get_me: GetMeUsecase::new(repo.clone()),
            update_profile: UpdateProfileUsecase::new(repo.clone()),
            list_users: ListUsersUsecase::new(repo.clone()),
            repo,
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_user_profile_workflow(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let email = format!("test-{}@example.com", Uuid::new_v4());
    let password = "password123".to_string();

    // 1. Register
    ctx.auth.register(email.clone(), password.clone()).await.unwrap();

    // 2. Login to get user (re-fetching via repo to get ID)
    let user = ctx.repo.find_by_email(&email).await.unwrap().unwrap();
    let user_id = user.id;

    // 3. Get Me
    let me = ctx.get_me.execute(user_id).await.unwrap();
    assert_eq!(me.email, email);

    // 4. Update Profile
    let new_email = format!("updated-{}@example.com", Uuid::new_v4());
    ctx.update_profile.execute(user_id, new_email.clone()).await.unwrap();

    // 5. Verify update
    let updated_me = ctx.get_me.execute(user_id).await.unwrap();
    assert_eq!(updated_me.email, new_email);

    // 6. List Users
    let users = ctx.list_users.execute().await.unwrap();
    assert!(!users.is_empty());
}
