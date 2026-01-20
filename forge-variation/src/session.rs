//! Session model for FORGE (v1).
//!
//! Sessions track the complete workflow from input through variation generation to approval.
//! They store base input, intent history, generated variations, and user approvals.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::{
    AssetClass, ParameterDeltaV1, ParameterSetV1, Seed, VariationSpecV1, PARAM_SCHEMA_VERSION,
};

/// Recommended file extension for saved sessions.
pub const SESSION_FILE_EXT: &str = "forge.json";

/// Source type for base silhouette input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaseInputType {
    Drawn,
    Image,
}

/// Reference to the base 2D input file (path-based for small sessions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseInputRefV1 {
    pub input_type: BaseInputType,
    pub source_path: String,
}

impl BaseInputRefV1 {
    /// Validate that the referenced path exists.
    pub fn validate(&self) -> Result<(), SessionError> {
        let path = Path::new(&self.source_path);
        if !path.exists() {
            tracing::error!(
                path = %self.source_path,
                "base input path does not exist"
            );
            return Err(SessionError::InvalidPath {
                path: self.source_path.clone(),
            });
        }
        Ok(())
    }
}

/// User intent entry (prompt) for a given iteration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentEntryV1 {
    pub iteration: u32,
    pub text: String,
}

/// Real-world dimensions in meters (Bevy/standard game engine units).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DimensionsMeters {
    pub height: f32,
    pub width: f32,
    pub depth: f32,
}

impl DimensionsMeters {
    /// Check that all dimensions are positive and finite.
    pub fn is_valid(&self) -> bool {
        self.height.is_finite()
            && self.width.is_finite()
            && self.depth.is_finite()
            && self.height > 0.0
            && self.width > 0.0
            && self.depth > 0.0
    }

    /// Convert to centimeters (for Unreal Engine export).
    pub fn to_centimeters(&self) -> DimensionsCm {
        DimensionsCm {
            height: self.height * 100.0,
            width: self.width * 100.0,
            depth: self.depth * 100.0,
        }
    }
}

/// Real-world dimensions in centimeters (for Unreal Engine compatibility).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DimensionsCm {
    pub height: f32,
    pub width: f32,
    pub depth: f32,
}

impl DimensionsCm {
    /// Check that all dimensions are positive and finite.
    pub fn is_valid(&self) -> bool {
        self.height.is_finite()
            && self.width.is_finite()
            && self.depth.is_finite()
            && self.height > 0.0
            && self.width > 0.0
            && self.depth > 0.0
    }

    /// Convert to meters (for Bevy/standard engines).
    pub fn to_meters(&self) -> DimensionsMeters {
        DimensionsMeters {
            height: self.height / 100.0,
            width: self.width / 100.0,
            depth: self.depth / 100.0,
        }
    }
}

/// Pivot point placement for engine integration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PivotMode {
    Center,
    BaseCenter,
}

/// Collision mesh generation mode.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollisionMode {
    None,
    Box,
    Convex,
}

/// Export settings for 3D asset generation.
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
            generate_lods: true, // Enable LODs by default for games
        }
    }
}

/// A single approved design ready for 3D generation and export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovedDesignV1 {
    pub approved_id: String,
    pub variation_id: String,
    pub dimensions: DimensionsMeters, // Primary unit: meters
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
    pub variations: Vec<VariationSpecV1>,
    pub approvals: Vec<ApprovedDesignV1>,
    pub notes: Option<String>,
}

impl SessionV1 {
    /// Create a new session. Returns error if base input path doesn't exist.
    pub fn new(
        asset_class: AssetClass,
        base_input: BaseInputRefV1,
        base_seed: Seed,
    ) -> Result<Self, SessionError> {
        tracing::info!(
            asset_class = ?asset_class,
            input_type = ?base_input.input_type,
            source_path = %base_input.source_path,
            base_seed = base_seed.0,
            "creating new session"
        );

        base_input.validate()?;

        let session_id = Uuid::new_v4();

        tracing::info!(
            session_id = %session_id,
            "session created successfully"
        );

        Ok(Self {
            session_id,
            asset_class,
            schema_version: PARAM_SCHEMA_VERSION.to_string(),
            base_input,
            base_seed,
            base_params: ParameterSetV1::default(),
            intent_history: vec![],
            variations: vec![],
            approvals: vec![],
            notes: None,
        })
    }

    /// Add an intent (user prompt). Returns iteration number. Rejects empty strings.
    pub fn push_intent(&mut self, text: impl Into<String>) -> Result<u32, SessionError> {
        let text = text.into();
        let trimmed = text.trim();

        if trimmed.is_empty() {
            tracing::warn!("rejecting empty intent text");
            return Err(SessionError::EmptyIntent);
        }

        let iter = self.intent_history.len() as u32;

        tracing::info!(
            iteration = iter,
            intent = %trimmed,
            "adding intent to history"
        );

        self.intent_history.push(IntentEntryV1 {
            iteration: iter,
            text,
        });

        Ok(iter)
    }

    /// Apply a parameter delta to base parameters (from AI or UI edits).
    pub fn apply_base_delta(&mut self, delta: &ParameterDeltaV1) {
        tracing::debug!("applying delta to base parameters");
        self.base_params.apply_delta(delta);
    }

    /// Generate variations, replacing current batch. Use append_variations() to keep existing.
    pub fn generate_variations(&mut self, count: usize, intent_text: impl Into<String>) {
        if !self.variations.is_empty() {
            tracing::warn!(
                existing_count = self.variations.len(),
                new_count = count,
                "generating new batch will replace existing variations"
            );
        }

        let batch = VariationSpecV1::generate_batch(
            self.session_id,
            self.asset_class.clone(),
            self.base_seed,
            self.base_params.clone(),
            intent_text,
            count,
        );

        tracing::info!(
            count = batch.len(),
            "variation batch generated and set as current"
        );

        self.variations = batch;
    }

    /// Append variations to existing batch without replacing.
    pub fn append_variations(&mut self, count: usize, intent_text: impl Into<String>) {
        let initial_count = self.variations.len();

        tracing::info!(
            existing = initial_count,
            appending = count,
            "appending variations to current batch"
        );

        let batch = VariationSpecV1::generate_batch(
            self.session_id,
            self.asset_class.clone(),
            self.base_seed,
            self.base_params.clone(),
            intent_text,
            count,
        );

        self.variations.extend(batch);

        tracing::info!(
            previous = initial_count,
            current = self.variations.len(),
            added = count,
            "variations appended successfully"
        );
    }

    /// Approve a variation with dimensions and export settings. Returns approval ID.
    /// Dimensions should be in meters (Bevy/standard units).
    pub fn approve_variation(
        &mut self,
        variation_id: &str,
        dimensions: DimensionsMeters,
        export: ExportSettingsV1,
        user_label: Option<String>,
    ) -> Result<String, SessionError> {
        tracing::debug!(
            variation_id = variation_id,
            dimensions = ?dimensions,
            "attempting to approve variation"
        );

        if !dimensions.is_valid() {
            tracing::error!(
                dimensions = ?dimensions,
                "invalid dimensions provided"
            );
            return Err(SessionError::InvalidDimensions);
        }

        let exists = self
            .variations
            .iter()
            .any(|v| v.variation_id == variation_id);

        if !exists {
            tracing::error!(
                variation_id = variation_id,
                available_count = self.variations.len(),
                "variation not found in current batch"
            );
            return Err(SessionError::UnknownVariation {
                variation_id: variation_id.to_string(),
            });
        }

        if self
            .approvals
            .iter()
            .any(|a| a.variation_id == variation_id)
        {
            tracing::warn!(variation_id = variation_id, "variation already approved");
            return Err(SessionError::DuplicateApproval {
                variation_id: variation_id.to_string(),
            });
        }

        let approved_id = format!("appr_{}_{}", self.approvals.len(), variation_id);

        tracing::info!(
            variation_id = variation_id,
            approved_id = %approved_id,
            dimensions_m = ?(dimensions.height, dimensions.width, dimensions.depth),
            user_label = ?user_label,
            "variation approved"
        );

        self.approvals.push(ApprovedDesignV1 {
            approved_id: approved_id.clone(),
            variation_id: variation_id.to_string(),
            dimensions,
            export,
            user_label,
        });

        Ok(approved_id)
    }

    /// Validate session internal consistency (schema version, references, duplicates, etc).
    pub fn validate(&self) -> Result<(), SessionError> {
        tracing::debug!(
            session_id = %self.session_id,
            "validating session integrity"
        );

        if self.schema_version != PARAM_SCHEMA_VERSION {
            tracing::error!(
                expected = PARAM_SCHEMA_VERSION,
                got = %self.schema_version,
                "schema version mismatch"
            );
            return Err(SessionError::SchemaVersionMismatch {
                expected: PARAM_SCHEMA_VERSION.to_string(),
                got: self.schema_version.clone(),
            });
        }

        self.base_input.validate()?;

        self.base_params.validate().map_err(|e| {
            tracing::error!(
                error = %e,
                "base parameters validation failed"
            );
            SessionError::InvalidParameters(e)
        })?;

        let mut seen_ids = HashSet::new();
        for var in &self.variations {
            if !seen_ids.insert(&var.variation_id) {
                tracing::error!(
                    variation_id = %var.variation_id,
                    "duplicate variation ID detected"
                );
                return Err(SessionError::DuplicateVariation {
                    variation_id: var.variation_id.clone(),
                });
            }
        }

        let variation_ids: HashSet<_> = self.variations.iter().map(|v| &v.variation_id).collect();
        for approval in &self.approvals {
            if !variation_ids.contains(&approval.variation_id) {
                tracing::error!(
                    approved_id = %approval.approved_id,
                    variation_id = %approval.variation_id,
                    "approval references non-existent variation"
                );
                return Err(SessionError::OrphanedApproval {
                    approved_id: approval.approved_id.clone(),
                    variation_id: approval.variation_id.clone(),
                });
            }

            if !approval.dimensions.is_valid() {
                tracing::error!(
                    approved_id = %approval.approved_id,
                    "approval has invalid dimensions"
                );
                return Err(SessionError::InvalidDimensions);
            }
        }

        tracing::debug!("session validation passed");
        Ok(())
    }
}

/// Session-level errors.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("dimensions must be positive finite numbers")]
    InvalidDimensions,

    #[error("unknown variation_id: {variation_id}")]
    UnknownVariation { variation_id: String },

    #[error("variation already approved: {variation_id}")]
    DuplicateApproval { variation_id: String },

    #[error("duplicate variation_id in batch: {variation_id}")]
    DuplicateVariation { variation_id: String },

    #[error("intent text cannot be empty")]
    EmptyIntent,

    #[error("schema version mismatch: expected {expected}, got {got}")]
    SchemaVersionMismatch { expected: String, got: String },

    #[error("invalid path: {path}")]
    InvalidPath { path: String },

    #[error("orphaned approval {approved_id} references non-existent variation {variation_id}")]
    OrphanedApproval {
        approved_id: String,
        variation_id: String,
    },

    #[error("parameter validation failed: {0}")]
    InvalidParameters(#[from] crate::ParamError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Save session to disk as pretty JSON. Validates before writing.
pub fn save_session(path: impl AsRef<Path>, session: &SessionV1) -> Result<(), SessionError> {
    let path = path.as_ref();

    tracing::info!(
        path = %path.display(),
        session_id = %session.session_id,
        "saving session"
    );

    session.validate()?;

    if let Some(parent) = path.parent() {
        tracing::debug!(
            parent = %parent.display(),
            "ensuring parent directory exists"
        );
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(session)?;
    let size_bytes = json.len();

    fs::write(path, &json)?;

    tracing::info!(
        path = %path.display(),
        size_bytes = size_bytes,
        variations = session.variations.len(),
        approvals = session.approvals.len(),
        "session saved successfully"
    );

    Ok(())
}

/// Load session from disk. Validates after reading.
pub fn load_session(path: impl AsRef<Path>) -> Result<SessionV1, SessionError> {
    let path = path.as_ref();

    tracing::info!(
        path = %path.display(),
        "loading session"
    );

    let data = fs::read_to_string(path)?;
    let size_bytes = data.len();

    tracing::debug!(size_bytes = size_bytes, "session file read");

    let session: SessionV1 = serde_json::from_str(&data)?;

    tracing::debug!(
        session_id = %session.session_id,
        schema_version = %session.schema_version,
        "session deserialized"
    );

    session.validate()?;

    tracing::info!(
        session_id = %session.session_id,
        variations = session.variations.len(),
        approvals = session.approvals.len(),
        intent_history = session.intent_history.len(),
        "session loaded successfully"
    );

    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimensions_validation() {
        let valid = DimensionsMeters {
            height: 2.5,
            width: 1.0,
            depth: 1.0,
        };
        assert!(valid.is_valid());

        let invalid_negative = DimensionsMeters {
            height: -1.0,
            width: 1.0,
            depth: 1.0,
        };
        assert!(!invalid_negative.is_valid());
    }

    #[test]
    fn test_meters_to_centimeters() {
        let meters = DimensionsMeters {
            height: 2.5,
            width: 1.0,
            depth: 0.5,
        };
        let cm = meters.to_centimeters();
        assert_eq!(cm.height, 250.0);
        assert_eq!(cm.width, 100.0);
        assert_eq!(cm.depth, 50.0);
    }

    #[test]
    fn test_centimeters_to_meters() {
        let cm = DimensionsCm {
            height: 250.0,
            width: 100.0,
            depth: 50.0,
        };
        let meters = cm.to_meters();
        assert_eq!(meters.height, 2.5);
        assert_eq!(meters.width, 1.0);
        assert_eq!(meters.depth, 0.5);
    }

    #[test]
    fn test_empty_intent_rejected() {
        let mut session = SessionV1 {
            session_id: Uuid::new_v4(),
            asset_class: AssetClass::ArenaProp,
            schema_version: PARAM_SCHEMA_VERSION.to_string(),
            base_input: BaseInputRefV1 {
                input_type: BaseInputType::Drawn,
                source_path: "test.png".into(),
            },
            base_seed: Seed(42),
            base_params: ParameterSetV1::default(),
            intent_history: vec![],
            variations: vec![],
            approvals: vec![],
            notes: None,
        };

        assert!(session.push_intent("").is_err());
        assert!(session.push_intent("   ").is_err());
        assert!(session.push_intent("valid intent").is_ok());
    }
}
