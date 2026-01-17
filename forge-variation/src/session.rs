//! Session model for FORGE (v1).
//!
//! Sessions are the durable unit of work. They store:
//! - base input reference (drawn/image)
//! - intent history
//! - generated variation specs (seeded + reproducible)
//! - user approvals (with real-world dimensions + export settings)

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::{
    AssetClass, ParameterDeltaV1, ParameterSetV1, Seed, VariationSpecV1, PARAM_SCHEMA_VERSION,
};

/// File extension recommended for saved sessions.
pub const SESSION_FILE_EXT: &str = "forge.json";

/// Where did the base silhouette come from?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaseInputType {
    Drawn,
    Image,
}

/// Reference to the base input. v1 keeps this as a path to keep sessions small.
/// (You can add an embedded-bytes option later if you want portable sessions.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseInputRefV1 {
    pub input_type: BaseInputType,
    pub source_path: String,
}

/// An intent entry represents what the user asked for at a given iteration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentEntryV1 {
    pub iteration: u32,
    pub text: String,
}

/// Real-world dimensions for the final 3D asset, in centimeters.
/// Unreal uses centimeters by default, so this keeps exports predictable.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DimensionsCm {
    pub height: f32,
    pub width: f32,
    pub depth: f32,
}

impl DimensionsCm {
    pub fn is_valid(&self) -> bool {
        self.height.is_finite()
            && self.width.is_finite()
            && self.depth.is_finite()
            && self.height > 0.0
            && self.width > 0.0
            && self.depth > 0.0
    }
}

/// Pivot placement hint for engine integration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PivotMode {
    Center,
    BaseCenter,
}

/// Collision generation hint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollisionMode {
    None,
    Box,
    Convex,
}

/// Export settings chosen at approval time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportSettingsV1 {
    pub pivot: PivotMode,
    pub collision: CollisionMode,
    pub generate_lods: bool,
}

impl Default for ExportSettingsV1 {
    fn default() -> Self {
        Self {
            pivot: PivotMode::BaseCenter,
            collision: CollisionMode::Box,
            generate_lods: false,
        }
    }
}

/// A single approval record: user selected a variation and provided dimensions/settings.
/// This is the exact unit that later becomes a packaged asset folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovedDesignV1 {
    pub approved_id: String,
    pub variation_id: String,
    pub dimensions_cm: DimensionsCm,
    pub export: ExportSettingsV1,
    pub user_label: Option<String>,
}

/// v1 session object. Save/load this as JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionV1 {
    pub session_id: Uuid,
    pub asset_class: AssetClass,
    pub schema_version: String,

    pub base_input: BaseInputRefV1,

    pub base_seed: Seed,
    pub base_params: ParameterSetV1,

    pub intent_history: Vec<IntentEntryV1>,

    /// The most recently generated variations for preview selection.
    pub variations: Vec<VariationSpecV1>,

    /// Approved designs ready for 3D generation + export.
    pub approvals: Vec<ApprovedDesignV1>,

    pub notes: Option<String>,
}

impl SessionV1 {
    /// Create a new session with defaults.
    pub fn new(asset_class: AssetClass, base_input: BaseInputRefV1, base_seed: Seed) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            asset_class,
            schema_version: PARAM_SCHEMA_VERSION.to_string(),
            base_input,
            base_seed,
            base_params: ParameterSetV1::default(),
            intent_history: vec![],
            variations: vec![],
            approvals: vec![],
            notes: None,
        }
    }

    /// Add an intent line (user prompt). Returns the iteration number.
    pub fn push_intent(&mut self, text: impl Into<String>) -> u32 {
        let iter = self.intent_history.len() as u32;
        self.intent_history.push(IntentEntryV1 {
            iteration: iter,
            text: text.into(),
        });
        iter
    }

    /// Apply a parameter delta to the session's base parameters (from UI edits or AI suggestions).
    pub fn apply_base_delta(&mut self, delta: &ParameterDeltaV1) {
        self.base_params.apply_delta(delta);
    }

    /// Generate a fresh batch of variations for preview.
    /// This overwrites `self.variations` to reflect the most recent generation.
    pub fn generate_variations(&mut self, count: usize, intent_text: impl Into<String>) {
        let batch = VariationSpecV1::generate_batch(
            self.session_id,
            self.asset_class.clone(),
            self.base_seed,
            self.base_params.clone(),
            intent_text,
            count,
        );
        self.variations = batch;
    }

    /// Approve a specific variation by ID, attaching dimensions + export settings.
    pub fn approve_variation(
        &mut self,
        variation_id: &str,
        dimensions_cm: DimensionsCm,
        export: ExportSettingsV1,
        user_label: Option<String>,
    ) -> Result<String, SessionError> {
        if !dimensions_cm.is_valid() {
            return Err(SessionError::InvalidDimensions);
        }

        let exists = self
            .variations
            .iter()
            .any(|v| v.variation_id == variation_id);
        if !exists {
            return Err(SessionError::UnknownVariation {
                variation_id: variation_id.to_string(),
            });
        }

        let approved_id = format!("appr_{}_{}", self.approvals.len(), variation_id);
        self.approvals.push(ApprovedDesignV1 {
            approved_id: approved_id.clone(),
            variation_id: variation_id.to_string(),
            dimensions_cm,
            export,
            user_label,
        });

        Ok(approved_id)
    }
}

/// Session-level errors keep it simple stupid.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("dimensions must be positive finite numbers")]
    InvalidDimensions,

    #[error("unknown variation_id: {variation_id}")]
    UnknownVariation { variation_id: String },
}

/// Save a session to disk as pretty JSON.
pub fn save_session(path: impl AsRef<Path>, session: &SessionV1) -> anyhow::Result<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        // fs::write does NOT create directories; tests may run with missing `target/`
        fs::create_dir_all(parent)
            .with_context(|| format!("create parent dir: {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(session).context("serialize session to json")?;
    fs::write(path, json).with_context(|| format!("write session file: {}", path.display()))?;
    Ok(())
}

/// Load a session from disk.
pub fn load_session(path: impl AsRef<Path>) -> anyhow::Result<SessionV1> {
    let path = path.as_ref();
    let data = fs::read_to_string(path)
        .with_context(|| format!("read session file: {}", path.display()))?;
    let session: SessionV1 = serde_json::from_str(&data).context("parse session json")?;
    Ok(session)
}
