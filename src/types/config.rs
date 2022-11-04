use k8s_openapi::api::core::v1::SecretKeySelector;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OdooConfig {
    pub without_demo: bool,
    pub list_database: bool,
    pub db_filter: Option<String>,
    pub admin_password: Option<SecretKeySelector>,
}
