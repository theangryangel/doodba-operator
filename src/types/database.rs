use k8s_openapi::api::core::v1::{ConfigMapKeySelector, SecretKeySelector};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    pub host: Option<ConfigMapKeySelector>,
    pub port: Option<ConfigMapKeySelector>,
    pub username: Option<SecretKeySelector>,
    pub password: Option<SecretKeySelector>,
    pub database: Option<String>,
}
