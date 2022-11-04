use super::annotations::Annotations;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileStore {
    /// Access mode
    pub access_modes: Option<Vec<String>>,
    /// Size
    pub size: Option<Quantity>,

    /// Storage Class Name
    pub storage_class_name: Option<String>,

    /// Existing claim
    pub existing_claim: Option<Vec<String>>,

    /// Annotations
    pub annotations: Annotations,
}
