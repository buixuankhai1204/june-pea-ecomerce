use crate::domain::user_repository::UserRepository;
use shared::error::AppError;
use std::sync::Arc;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct MembershipSummary {
    pub total_members: usize,
    pub tier_counts: HashMap<String, usize>,
}

pub struct GetMembershipSummaryUsecase {
    repo: Arc<dyn UserRepository>,
}

impl GetMembershipSummaryUsecase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<MembershipSummary, AppError> {
        let members = self.repo.list_memberships().await?;
        let total_members = members.len();
        let mut tier_counts = HashMap::new();
        for m in members {
            *tier_counts.entry(m.tier).or_insert(0) += 1;
        }
        Ok(MembershipSummary { total_members, tier_counts })
    }
}
