use schemars::JsonSchema;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum DoodbaPhase {
    Pending,   // initial state
    Creating,  // Running before_create
    Upgrading, // Running any before_update
    Running,   // healthy
    Failed,    // unhealthy installation
    Suspended,
}

impl Default for DoodbaPhase {
    fn default() -> Self {
        DoodbaPhase::Pending
    }
}
