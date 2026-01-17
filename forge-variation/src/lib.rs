//! forge-variation: deterministic parameter schema + variation specs for FORGE.
//!
//! Design rules:
//! - Parameters are bounded and clamped.
//! - Variation is deterministic: base_seed + index -> seed.
//! - AI never touches geometry; it only suggests parameter deltas.
//! - All structs are serializable for session save/load.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Schema version for forward compatibility.
pub const PARAM_SCHEMA_VERSION: &str = "1.0";

/// A deterministic seed used to reproduce variations.
/// We keep it simple: u64, no RNG dependency yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Seed(pub u64);

impl Seed {
    /// Derive a new seed deterministically from a base seed and an index.
    /// This is intentionally simple and stable.
    pub fn derive(self, index: u64) -> Seed {
        // SplitMix64-ish mixing (stable, fast, no deps)
        let mut z = self.0.wrapping_add(0x9E3779B97F4A7C15).wrapping_add(index);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        Seed(z ^ (z >> 31))
    }
}

/// High-level asset categories to constrain parameters & generation rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    ArenaProp,
    ArenaWall,
    Pillar,
    Debris,
    // Keep this list small in v1; extend later.
}

/// A bounded scalar parameter.
/// v1 keeps everything as f32 for simplicity; we can type-split later.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounded {
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

impl Bounded {
    pub fn new(value: f32, min: f32, max: f32) -> Result<Self, ParamError> {
        if !(min < max) {
            return Err(ParamError::InvalidBounds { min, max });
        }
        Ok(Self { value, min, max }.clamped())
    }

    pub fn clamped(mut self) -> Self {
        if self.value < self.min {
            self.value = self.min;
        } else if self.value > self.max {
            self.value = self.max;
        }
        self
    }
}

/// The canonical parameter set for v1 generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSetV1 {
    /// Scales the silhouette height in the final asset.
    pub height_scale: Bounded, // [0.5, 2.0]
    /// Depth of extrusion for 2.5D -> 3D.
    pub extrusion_depth: Bounded, // [0.1, 1.0]
    /// Softens edges.
    pub bevel_amount: Bounded, // [0.0, 0.5]
    /// How strongly symmetry is broken.
    pub symmetry_break: Bounded, // [0.0, 1.0]
    /// Wear/damage intensity.
    pub erosion_intensity: Bounded, // [0.0, 1.0]
    /// Fine detail variation.
    pub detail_density: Bounded, // [0.0, 1.0]
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
    /// Clamp all values to their bounds.
    pub fn clamp_all(mut self) -> Self {
        self.height_scale = self.height_scale.clamped();
        self.extrusion_depth = self.extrusion_depth.clamped();
        self.bevel_amount = self.bevel_amount.clamped();
        self.symmetry_break = self.symmetry_break.clamped();
        self.erosion_intensity = self.erosion_intensity.clamped();
        self.detail_density = self.detail_density.clamped();
        self
    }

    /// Apply a delta (suggested by AI or user edits). Deltas are additive and then clamped.
    pub fn apply_delta(&mut self, delta: &ParameterDeltaV1) {
        if let Some(v) = delta.height_scale {
            self.height_scale.value += v;
        }
        if let Some(v) = delta.extrusion_depth {
            self.extrusion_depth.value += v;
        }
        if let Some(v) = delta.bevel_amount {
            self.bevel_amount.value += v;
        }
        if let Some(v) = delta.symmetry_break {
            self.symmetry_break.value += v;
        }
        if let Some(v) = delta.erosion_intensity {
            self.erosion_intensity.value += v;
        }
        if let Some(v) = delta.detail_density {
            self.detail_density.value += v;
        }
        *self = self.clone().clamp_all();
    }
}

/// Sparse additive deltas to parameters (AI output should map to this).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ParameterDeltaV1 {
    pub height_scale: Option<f32>,
    pub extrusion_depth: Option<f32>,
    pub bevel_amount: Option<f32>,
    pub symmetry_break: Option<f32>,
    pub erosion_intensity: Option<f32>,
    pub detail_density: Option<f32>,
}

/// A single variation request/spec. This is the unit that becomes a 2D ref (and later a 3D asset).
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
    /// Create N deterministic variations from a base seed + base params.
    pub fn generate_batch(
        base_session_id: Uuid,
        asset_class: AssetClass,
        base_seed: Seed,
        base_params: ParameterSetV1,
        intent_text: impl Into<String>,
        count: usize,
    ) -> Vec<Self> {
        let intent_text = intent_text.into();
        (0..count)
            .map(|i| {
                let seed = base_seed.derive(i as u64);
                let variation_id = format!("var_{:04}_{}", i, seed.0);
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
            .collect()
    }
}

/// Errors related to parameter schema and validation.
#[derive(Debug, Error)]
pub enum ParamError {
    #[error("invalid bounds: min={min} max={max}")]
    InvalidBounds { min: f32, max: f32 },
}

pub mod session;

pub use session::{
    ApprovedDesignV1, BaseInputRefV1, BaseInputType, CollisionMode, DimensionsCm, ExportSettingsV1,
    IntentEntryV1, PivotMode, SessionError, SessionV1, SESSION_FILE_EXT,
};
