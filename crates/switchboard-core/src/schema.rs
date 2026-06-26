use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSchema {
    pub resource: String,
    pub required: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
}
