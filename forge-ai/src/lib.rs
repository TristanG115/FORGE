// This defines the JSON format that we accept from the AI model
// Parse and validate it
// Convert the stuff here to what the rest of FORGE can understand

//import
use forge_variation::ParameterDeltaV1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiResponseV1 {
    pub adjustments: ParameterDeltaV1,
    pub confidence: Option<f32>,
    pub notes: Option<String>,
}

pub struct AiTelemetryV1 {
    pub model_name: String,
    pub time_taken_s: f32,
    pub version: String,
    pub warnings: Vec<String>,
}
