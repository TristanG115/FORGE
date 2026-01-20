//! forge-variation: deterministic parameter schema + variation specs for FORGE.
//!
//! This module provides the core parameter system for FORGE's deterministic asset generation.
//! It defines parameters, variations, and sessions for the 2D-to-3D asset pipeline.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Schema version for forward compatibility.
pub const PARAM_SCHEMA_VERSION: &str = "1.0";

/// Deterministic seed for variation generation. Use derive() to create child seeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Seed(pub u64);

impl Seed {
    /// Derive a new seed deterministically from this seed and an index.
    /// Uses SplitMix64-style mixing for stable, well-distributed results.
    pub fn derive(self, index: u64) -> Seed {
        let mut z = self.0.wrapping_add(0x9E3779B97F4A7C15).wrapping_add(index);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        let result = z ^ (z >> 31);

        tracing::trace!(
            base_seed = self.0,
            index = index,
            derived_seed = result,
            "derived seed from base"
        );

        Seed(result)
    }
}

/// High-level asset categories for parameter constraints and generation rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    ArenaProp,
    ArenaWall,
    Pillar,
    Debris,
}

/// Bounded parameter with automatic clamping to [min, max].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounded {
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

impl Bounded {
    /// Create a bounded parameter, clamping value to [min, max]. Returns error if min >= max.
    pub fn new(value: f32, min: f32, max: f32) -> Result<Self, ParamError> {
        if !(min < max) {
            tracing::error!(
                min = min,
                max = max,
                "invalid bounds: min must be less than max"
            );
            return Err(ParamError::InvalidBounds { min, max });
        }

        let bounded = Self { value, min, max };
        let clamped = bounded.clamped();

        if clamped.value != value {
            tracing::debug!(
                original = value,
                clamped = clamped.value,
                min = min,
                max = max,
                "value clamped to valid range"
            );
        }

        Ok(clamped)
    }

    /// Return a copy with value clamped to [min, max].
    #[must_use]
    pub fn clamped(mut self) -> Self {
        if self.value < self.min {
            self.value = self.min;
        } else if self.value > self.max {
            self.value = self.max;
        }
        self
    }

    /// Set a new value, clamping to bounds if necessary.
    pub fn set(&mut self, new_value: f32) {
        let old = self.value;
        self.value = new_value;
        *self = self.clamped();

        if self.value != new_value {
            tracing::debug!(
                attempted = new_value,
                clamped = self.value,
                min = self.min,
                max = self.max,
                "value clamped during set"
            );
        } else if self.value != old {
            tracing::trace!(old = old, new = self.value, "parameter value updated");
        }
    }
}

/// Complete set of generation parameters for v1.
/// All parameters are bounded and will automatically clamp values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSetV1 {
    pub height_scale: Bounded,      // [0.5, 2.0] - Scales silhouette height
    pub extrusion_depth: Bounded,   // [0.1, 1.0] - Depth of 2.5D → 3D extrusion
    pub bevel_amount: Bounded,      // [0.0, 0.5] - Edge softening
    pub symmetry_break: Bounded,    // [0.0, 1.0] - How much symmetry is broken
    pub erosion_intensity: Bounded, // [0.0, 1.0] - Wear/damage intensity
    pub detail_density: Bounded,    // [0.0, 1.0] - Fine detail variation
}

impl Default for ParameterSetV1 {
    fn default() -> Self {
        Self {
            height_scale: Bounded {
                value: 1.0,
                min: 0.5,
                max: 2.0,
            },
            extrusion_depth: Bounded {
                value: 0.5,
                min: 0.1,
                max: 1.0,
            },
            bevel_amount: Bounded {
                value: 0.10,
                min: 0.0,
                max: 0.5,
            },
            symmetry_break: Bounded {
                value: 0.0,
                min: 0.0,
                max: 1.0,
            },
            erosion_intensity: Bounded {
                value: 0.0,
                min: 0.0,
                max: 1.0,
            },
            detail_density: Bounded {
                value: 0.20,
                min: 0.0,
                max: 1.0,
            },
        }
    }
}

impl ParameterSetV1 {
    /// Clamp all parameter values to their bounds. Use after deserialization or manual modification.
    #[must_use]
    pub fn clamp_all(mut self) -> Self {
        self.height_scale = self.height_scale.clamped();
        self.extrusion_depth = self.extrusion_depth.clamped();
        self.bevel_amount = self.bevel_amount.clamped();
        self.symmetry_break = self.symmetry_break.clamped();
        self.erosion_intensity = self.erosion_intensity.clamped();
        self.detail_density = self.detail_density.clamped();
        self
    }

    /// Apply a delta (from AI or user edits). Deltas are additive, then clamped.
    pub fn apply_delta(&mut self, delta: &ParameterDeltaV1) {
        tracing::debug!("applying parameter delta");

        let mut changes = 0;

        if let Some(v) = delta.height_scale {
            let old = self.height_scale.value;
            self.height_scale.value += v;
            self.height_scale = self.height_scale.clamped();
            if self.height_scale.value != old {
                tracing::trace!(
                    field = "height_scale",
                    old = old,
                    delta = v,
                    new = self.height_scale.value,
                    "parameter adjusted"
                );
                changes += 1;
            }
        }

        if let Some(v) = delta.extrusion_depth {
            let old = self.extrusion_depth.value;
            self.extrusion_depth.value += v;
            self.extrusion_depth = self.extrusion_depth.clamped();
            if self.extrusion_depth.value != old {
                tracing::trace!(
                    field = "extrusion_depth",
                    old = old,
                    delta = v,
                    new = self.extrusion_depth.value,
                    "parameter adjusted"
                );
                changes += 1;
            }
        }

        if let Some(v) = delta.bevel_amount {
            let old = self.bevel_amount.value;
            self.bevel_amount.value += v;
            self.bevel_amount = self.bevel_amount.clamped();
            if self.bevel_amount.value != old {
                tracing::trace!(
                    field = "bevel_amount",
                    old = old,
                    delta = v,
                    new = self.bevel_amount.value,
                    "parameter adjusted"
                );
                changes += 1;
            }
        }

        if let Some(v) = delta.symmetry_break {
            let old = self.symmetry_break.value;
            self.symmetry_break.value += v;
            self.symmetry_break = self.symmetry_break.clamped();
            if self.symmetry_break.value != old {
                tracing::trace!(
                    field = "symmetry_break",
                    old = old,
                    delta = v,
                    new = self.symmetry_break.value,
                    "parameter adjusted"
                );
                changes += 1;
            }
        }

        if let Some(v) = delta.erosion_intensity {
            let old = self.erosion_intensity.value;
            self.erosion_intensity.value += v;
            self.erosion_intensity = self.erosion_intensity.clamped();
            if self.erosion_intensity.value != old {
                tracing::trace!(
                    field = "erosion_intensity",
                    old = old,
                    delta = v,
                    new = self.erosion_intensity.value,
                    "parameter adjusted"
                );
                changes += 1;
            }
        }

        if let Some(v) = delta.detail_density {
            let old = self.detail_density.value;
            self.detail_density.value += v;
            self.detail_density = self.detail_density.clamped();
            if self.detail_density.value != old {
                tracing::trace!(
                    field = "detail_density",
                    old = old,
                    delta = v,
                    new = self.detail_density.value,
                    "parameter adjusted"
                );
                changes += 1;
            }
        }

        tracing::debug!(changes = changes, "delta application complete");
    }

    /// Validate all parameters are within bounds. Should always pass if constructed properly.
    pub fn validate(&self) -> Result<(), ParamError> {
        let params = [
            ("height_scale", &self.height_scale),
            ("extrusion_depth", &self.extrusion_depth),
            ("bevel_amount", &self.bevel_amount),
            ("symmetry_break", &self.symmetry_break),
            ("erosion_intensity", &self.erosion_intensity),
            ("detail_density", &self.detail_density),
        ];

        for (name, param) in params {
            if param.value < param.min || param.value > param.max {
                tracing::error!(
                    field = name,
                    value = param.value,
                    min = param.min,
                    max = param.max,
                    "parameter out of bounds"
                );
                return Err(ParamError::OutOfRange {
                    field: name.to_string(),
                    value: param.value,
                    min: param.min,
                    max: param.max,
                });
            }
        }

        Ok(())
    }
}

/// Sparse additive deltas to parameters. AI output maps to this. Only set fields that should change.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ParameterDeltaV1 {
    pub height_scale: Option<f32>,
    pub extrusion_depth: Option<f32>,
    pub bevel_amount: Option<f32>,
    pub symmetry_break: Option<f32>,
    pub erosion_intensity: Option<f32>,
    pub detail_density: Option<f32>,
}

/// A single variation spec. Deterministic: same spec always produces same output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariationSpecV1 {
    pub variation_id: String,
    pub base_session_id: Uuid,
    pub asset_class: AssetClass,
    pub schema_version: String,
    pub seed: Seed,
    pub params: ParameterSetV1,
    pub intent_text: String,
}

impl VariationSpecV1 {
    /// Generate N deterministic variations from base seed and params.
    /// Each variation gets a unique seed derived from base_seed.derive(index).
    pub fn generate_batch(
        base_session_id: Uuid,
        asset_class: AssetClass,
        base_seed: Seed,
        base_params: ParameterSetV1,
        intent_text: impl Into<String>,
        count: usize,
    ) -> Vec<Self> {
        let intent_text = intent_text.into();

        if intent_text.trim().is_empty() {
            tracing::warn!("empty intent text provided for variation batch");
        }

        tracing::info!(
            session_id = %base_session_id,
            asset_class = ?asset_class,
            base_seed = base_seed.0,
            count = count,
            "generating variation batch"
        );

        let variations: Vec<_> = (0..count)
            .map(|i| {
                let seed = base_seed.derive(i as u64);
                let variation_id = format!("var_{:04}_{}", i, seed.0);

                tracing::debug!(
                    index = i,
                    variation_id = %variation_id,
                    seed = seed.0,
                    "generated variation spec"
                );

                Self {
                    variation_id,
                    base_session_id,
                    asset_class: asset_class.clone(),
                    schema_version: PARAM_SCHEMA_VERSION.to_string(),
                    seed,
                    params: base_params.clone(),
                    intent_text: intent_text.clone(),
                }
            })
            .collect();

        tracing::info!(
            count = variations.len(),
            "variation batch generated successfully"
        );

        variations
    }
}

/// Parameter-related errors.
#[derive(Debug, Error)]
pub enum ParamError {
    #[error("invalid bounds for parameter: min={min} max={max}")]
    InvalidBounds { min: f32, max: f32 },

    #[error("parameter '{field}' out of range: value={value}, valid range=[{min}, {max}]")]
    OutOfRange {
        field: String,
        value: f32,
        min: f32,
        max: f32,
    },
}

// Module declarations
pub mod export;
pub mod project;
pub mod session;

// Re-export session types
pub use session::{
    load_session, save_session, ApprovedDesignV1, BaseInputRefV1, BaseInputType, CollisionMode,
    DimensionsCm, ExportSettingsV1, IntentEntryV1, PivotMode, SessionError, SessionV1,
    SESSION_FILE_EXT,
};

// Re-export export types
pub use export::{
    Axis, ExportConfig, ExportError, ExportFormat, LodConfig, MaterialConfig, MaterialSystem,
    NamingConfig, TargetEngine,
};

// Re-export project types <- NEW: Export project types
pub use project::{
    AestheticProfile, AssetReference, ColorPalette, Project, ProjectError, ProjectStyleProfile,
    TextureStyle,
};
