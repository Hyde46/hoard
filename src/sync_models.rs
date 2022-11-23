use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub id: u32,
    pub token: String,
    pub user_id: u32,
    pub revoked: bool,
    pub created_at: String,
}
