use ulid::Ulid;

use crate::dto;

pub trait UserQueryService {
    async fn get_summary(&self, id: Ulid) -> anyhow::Result<Option<dto::UserSummary>>;
}

pub trait AuthService: Send + Sync {
    fn hash_password(&self, password: &str) -> anyhow::Result<String>;
    fn verify_password(&self, password: &str, phc: &str) -> anyhow::Result<bool>;
}
