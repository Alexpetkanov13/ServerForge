use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaRuntime {
    pub id: String,
    pub path: String,
    pub version: String,
    pub major: u32,
    pub vendor: String,
    pub architecture: String,
    pub is_managed: bool,
    pub compatible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaTestResult {
    pub ok: bool,
    pub version: String,
    pub vendor: String,
    pub architecture: String,
    pub output: String,
}
