use schemars::JsonSchema;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
pub enum DoodbaPhase {
    #[default]
    Initializing,
    Upgrading,
    Ready,
    Suspended,
}
