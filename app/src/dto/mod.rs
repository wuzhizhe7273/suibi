pub mod iam;
pub mod post;
pub mod tag;
pub mod taxonomy;
use ulid::Ulid;

pub struct UserSummary {
    pub id: Ulid,
    pub username: String,
    pub email: Option<String>,
}
