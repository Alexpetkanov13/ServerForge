use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyField {
    pub key: String,
    pub label: String,
    pub value: String,
    pub kind: String,
    pub options: Vec<String>,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerProperties {
    pub fields: Vec<PropertyField>,
    pub raw: String,
}
