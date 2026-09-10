use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BackupRecord {
    pub id: String,
    pub server_id: String,
    pub label: Option<String>,
    pub path: String,
    pub size_bytes: i64,
    pub trigger: String,
    pub created_at: String,
}
