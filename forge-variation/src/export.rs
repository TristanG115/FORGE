//! Asset export configuration and validation for FORGE.
//!
//! Defines export configurations for converting approved variations into game engine assets.
//! Primary target: Bevy game engine (Rust-based, uses GLTF format).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Supported 3D export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Gltf, // glTF 2.0 binary (primary for Bevy)
    Obj,  // Wavefront OBJ (simple meshes)
    Fbx,  // Autodesk FBX (for other engines)
}

impl ExportFormat {
    /// Get file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Gltf => "glb",
            ExportFormat::Obj => "obj",
            ExportFormat::Fbx => "fbx",
        }
    }

    /// Check if this format supports LOD (Level of Detail).
    pub fn supports_lod(&self) -> bool {
        match self {
            ExportFormat::Gltf => true,
            ExportFormat::Obj => false,
            ExportFormat::Fbx => true,
        }
    }

    /// Check if this format supports embedded materials.
    pub fn supports_materials(&self) -> bool {
        match self {
            ExportFormat::Gltf => true,
            ExportFormat::Obj => true,
            ExportFormat::Fbx => true,
        }
    }
}

/// Target game engine for export optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetEngine {
    Bevy,          // Bevy game engine (primary target)
    UnrealEngine5, // Unreal Engine 5.x (legacy support)
    UnrealEngine4, // Unreal Engine 4.x (legacy support)
    Unity,         // Unity engine
    Generic,       // Generic export
}

impl TargetEngine {
    /// Get recommended up-axis for this engine.
    pub fn up_axis(&self) -> Axis {
        match self {
            TargetEngine::Bevy => Axis::Y,  // Y-up (standard)
            TargetEngine::Unity => Axis::Y, // Y-up
            TargetEngine::UnrealEngine5 | TargetEngine::UnrealEngine4 => Axis::Z, // Z-up
            TargetEngine::Generic => Axis::Y, // Default Y-up
        }
    }

    /// Get recommended unit scale. Returns (scale_factor, unit_name).
    /// Bevy uses meters, Unreal uses centimeters.
    pub fn unit_info(&self) -> (f32, &'static str) {
        match self {
            TargetEngine::Bevy => (1.0, "meters"),
            TargetEngine::Unity => (1.0, "meters"),
            TargetEngine::UnrealEngine5 | TargetEngine::UnrealEngine4 => (100.0, "centimeters"),
            TargetEngine::Generic => (1.0, "meters"),
        }
    }

    /// Check if engine uses right-handed coordinate system.
    pub fn is_right_handed(&self) -> bool {
        match self {
            TargetEngine::Bevy => true,
            TargetEngine::Unity => true,
            TargetEngine::UnrealEngine5 | TargetEngine::UnrealEngine4 => false, // Left-handed
            TargetEngine::Generic => true,
        }
    }
}

/// Coordinate system axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Axis {
    X,
    Y,
    Z,
}

/// LOD (Level of Detail) generation configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LodConfig {
    pub level_count: u32,              // Number of LOD levels (0 = just base mesh)
    pub reduction_factor: f32,         // Triangle reduction per level (0.0 to 1.0)
    pub min_triangle_count: u32,       // Minimum triangles for lowest LOD
    pub distance_thresholds: Vec<f32>, // Distance in meters/units for LOD switching
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            level_count: 3,
            reduction_factor: 0.5,
            min_triangle_count: 100,
            // Distance thresholds in meters for Bevy
            distance_thresholds: vec![0.0, 10.0, 30.0, 100.0], // LOD0, LOD1, LOD2, LOD3
        }
    }
}

impl LodConfig {
    /// Create LOD config optimized for Bevy (distance-based).
    pub fn for_bevy() -> Self {
        Self {
            level_count: 3,
            reduction_factor: 0.5,
            min_triangle_count: 100,
            distance_thresholds: vec![0.0, 15.0, 50.0, 150.0],
        }
    }

    /// Validate LOD configuration.
    pub fn validate(&self) -> Result<(), ExportError> {
        if self.level_count > 10 {
            tracing::warn!(
                level_count = self.level_count,
                "unusually high LOD count (max recommended: 10)"
            );
        }

        if self.reduction_factor <= 0.0 || self.reduction_factor >= 1.0 {
            tracing::error!(
                reduction_factor = self.reduction_factor,
                "reduction factor must be in (0.0, 1.0)"
            );
            return Err(ExportError::InvalidLodConfig {
                reason: format!(
                    "reduction_factor {} must be in (0.0, 1.0)",
                    self.reduction_factor
                ),
            });
        }

        if self.min_triangle_count == 0 {
            tracing::error!("min_triangle_count cannot be zero");
            return Err(ExportError::InvalidLodConfig {
                reason: "min_triangle_count must be > 0".into(),
            });
        }

        // Validate distance thresholds are in ascending order
        for window in self.distance_thresholds.windows(2) {
            if window[0] >= window[1] {
                tracing::error!(
                    distance_thresholds = ?self.distance_thresholds,
                    "distance thresholds must be in ascending order"
                );
                return Err(ExportError::InvalidLodConfig {
                    reason: "distance_thresholds must be in ascending order".into(),
                });
            }
        }

        Ok(())
    }
}

/// Material system for export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialSystem {
    Pbr,    // Physically Based Rendering (standard for Bevy)
    Legacy, // Simple diffuse/specular
}

/// Material export configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialConfig {
    pub system: MaterialSystem,
    pub generate_textures: bool,
    pub texture_resolution: u32, // Must be power of 2
    pub generate_normal_maps: bool,
    pub generate_ao_maps: bool,
    pub generate_metallic_roughness: bool, // Combined texture for PBR
    pub base_color: Option<[f32; 3]>,      // RGB in 0.0-1.0 (if not using textures)
    pub roughness: f32,                    // For PBR (0.0-1.0)
    pub metallic: f32,                     // For PBR (0.0-1.0)
}

impl Default for MaterialConfig {
    fn default() -> Self {
        Self {
            system: MaterialSystem::Pbr,
            generate_textures: true,
            texture_resolution: 2048,
            generate_normal_maps: true,
            generate_ao_maps: true,
            generate_metallic_roughness: true, // Standard for GLTF/Bevy
            base_color: None,
            roughness: 0.7,
            metallic: 0.0,
        }
    }
}

impl MaterialConfig {
    /// Create config optimized for Bevy (PBR workflow).
    pub fn for_bevy() -> Self {
        Self {
            system: MaterialSystem::Pbr,
            generate_textures: true,
            texture_resolution: 1024, // Good balance for games
            generate_normal_maps: true,
            generate_ao_maps: true,
            generate_metallic_roughness: true,
            base_color: None,
            roughness: 0.7,
            metallic: 0.0,
        }
    }

    /// Validate material configuration.
    pub fn validate(&self) -> Result<(), ExportError> {
        if self.generate_textures {
            if !self.texture_resolution.is_power_of_two() {
                tracing::error!(
                    resolution = self.texture_resolution,
                    "texture resolution must be power of 2"
                );
                return Err(ExportError::InvalidMaterialConfig {
                    reason: format!(
                        "texture_resolution {} is not a power of 2",
                        self.texture_resolution
                    ),
                });
            }

            if self.texture_resolution < 256 || self.texture_resolution > 8192 {
                tracing::warn!(
                    resolution = self.texture_resolution,
                    "unusual texture resolution (recommended: 512-2048 for games)"
                );
            }
        }

        if self.roughness < 0.0 || self.roughness > 1.0 {
            tracing::error!(
                roughness = self.roughness,
                "roughness must be in [0.0, 1.0]"
            );
            return Err(ExportError::InvalidMaterialConfig {
                reason: format!("roughness {} must be in [0.0, 1.0]", self.roughness),
            });
        }

        if self.metallic < 0.0 || self.metallic > 1.0 {
            tracing::error!(metallic = self.metallic, "metallic must be in [0.0, 1.0]");
            return Err(ExportError::InvalidMaterialConfig {
                reason: format!("metallic {} must be in [0.0, 1.0]", self.metallic),
            });
        }

        if let Some(color) = self.base_color {
            for (i, &channel) in color.iter().enumerate() {
                if !(0.0..=1.0).contains(&channel) {
                    tracing::error!(
                        channel = i,
                        value = channel,
                        "base_color channel out of range"
                    );
                    return Err(ExportError::InvalidMaterialConfig {
                        reason: format!("base_color[{}] = {} must be in [0.0, 1.0]", i, channel),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Asset naming conventions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamingConfig {
    pub prefix: String,
    pub include_session_id: bool,
    pub include_variation_id: bool,
    pub separator: String,
    pub lowercase: bool,
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            prefix: String::new(), // No prefix for Bevy (simpler)
            include_session_id: false,
            include_variation_id: true,
            separator: "_".into(),
            lowercase: true, // Rust convention
        }
    }
}

impl NamingConfig {
    /// Create naming config for Bevy (lowercase, no prefix).
    pub fn for_bevy() -> Self {
        Self {
            prefix: String::new(),
            include_session_id: false,
            include_variation_id: true,
            separator: "_".into(),
            lowercase: true,
        }
    }

    /// Generate filename based on this config.
    pub fn generate_filename(
        &self,
        user_label: &str,
        variation_id: &str,
        extension: &str,
    ) -> String {
        let mut parts = Vec::new();

        if !self.prefix.is_empty() {
            parts.push(self.prefix.clone());
        }

        if !user_label.is_empty() {
            parts.push(user_label.to_string());
        }

        if self.include_variation_id {
            parts.push(variation_id.to_string());
        }

        let mut name = parts.join(&self.separator);

        if self.lowercase {
            name = name.to_lowercase();
        }

        format!("{}.{}", name, extension)
    }

    /// Validate naming configuration (checks for invalid filename characters).
    pub fn validate(&self) -> Result<(), ExportError> {
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        for ch in invalid_chars {
            if self.prefix.contains(ch) {
                tracing::error!(
                    prefix = %self.prefix,
                    invalid_char = %ch,
                    "prefix contains invalid filename character"
                );
                return Err(ExportError::InvalidNamingConfig {
                    reason: format!("prefix contains invalid character '{}'", ch),
                });
            }
        }

        if self.separator.len() > 5 {
            tracing::warn!(
                separator = %self.separator,
                "unusually long separator (recommended: 1-2 chars)"
            );
        }

        Ok(())
    }
}

/// Complete export configuration for the export pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub target_engine: TargetEngine,
    pub lod_config: Option<LodConfig>,
    pub material_config: MaterialConfig,
    pub naming: NamingConfig,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self::bevy() // Default to Bevy
    }
}

impl ExportConfig {
    /// Create preset for Bevy game engine (primary target).
    pub fn bevy() -> Self {
        tracing::debug!("creating Bevy export config preset");
        Self {
            format: ExportFormat::Gltf,
            target_engine: TargetEngine::Bevy,
            lod_config: Some(LodConfig::for_bevy()),
            material_config: MaterialConfig::for_bevy(),
            naming: NamingConfig::for_bevy(),
        }
    }

    /// Create preset for Unreal Engine 5 (legacy support).
    pub fn unreal_engine_5() -> Self {
        tracing::debug!("creating Unreal Engine 5 export config preset");
        Self {
            format: ExportFormat::Fbx,
            target_engine: TargetEngine::UnrealEngine5,
            lod_config: Some(LodConfig::default()),
            material_config: MaterialConfig::default(),
            naming: NamingConfig {
                prefix: "SM".into(), // Unreal convention for Static Mesh
                lowercase: false,
                ..Default::default()
            },
        }
    }

    /// Create preset for Unity engine.
    pub fn unity() -> Self {
        tracing::debug!("creating Unity export config preset");
        Self {
            format: ExportFormat::Fbx,
            target_engine: TargetEngine::Unity,
            lod_config: Some(LodConfig::default()),
            material_config: MaterialConfig::default(),
            naming: NamingConfig::default(),
        }
    }

    /// Create preset for web/preview (GLTF, no LODs, smaller textures).
    pub fn web_preview() -> Self {
        tracing::debug!("creating web preview export config preset");
        Self {
            format: ExportFormat::Gltf,
            target_engine: TargetEngine::Generic,
            lod_config: None,
            material_config: MaterialConfig {
                texture_resolution: 1024,
                ..Default::default()
            },
            naming: NamingConfig {
                prefix: "preview".into(),
                lowercase: true,
                ..Default::default()
            },
        }
    }

    /// Validate entire export configuration.
    pub fn validate(&self) -> Result<(), ExportError> {
        tracing::debug!("validating export configuration");

        if let Some(ref lod_config) = self.lod_config {
            if !self.format.supports_lod() {
                tracing::error!(
                    format = ?self.format,
                    "format does not support LODs"
                );
                return Err(ExportError::IncompatibleSettings {
                    reason: format!("{:?} format does not support LOD generation", self.format),
                });
            }
            lod_config.validate()?;
        }

        self.material_config.validate()?;
        self.naming.validate()?;

        tracing::debug!("export configuration validated successfully");
        Ok(())
    }

    /// Get output file path for an asset.
    pub fn get_output_path(
        &self,
        base_dir: impl AsRef<Path>,
        user_label: &str,
        variation_id: &str,
    ) -> PathBuf {
        let filename =
            self.naming
                .generate_filename(user_label, variation_id, self.format.extension());

        base_dir.as_ref().join(filename)
    }
}

/// Export configuration errors.
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("invalid LOD configuration: {reason}")]
    InvalidLodConfig { reason: String },

    #[error("invalid material configuration: {reason}")]
    InvalidMaterialConfig { reason: String },

    #[error("invalid naming configuration: {reason}")]
    InvalidNamingConfig { reason: String },

    #[error("incompatible export settings: {reason}")]
    IncompatibleSettings { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bevy_preset() {
        let config = ExportConfig::bevy();
        assert_eq!(config.format, ExportFormat::Gltf);
        assert_eq!(config.target_engine, TargetEngine::Bevy);
        assert!(config.validate().is_ok());
        assert_eq!(config.target_engine.up_axis(), Axis::Y);
        assert!(config.target_engine.is_right_handed());
    }

    #[test]
    fn test_bevy_uses_meters() {
        let config = ExportConfig::bevy();
        let (scale, unit) = config.target_engine.unit_info();
        assert_eq!(scale, 1.0);
        assert_eq!(unit, "meters");
    }

    #[test]
    fn test_lod_distance_thresholds() {
        let lod = LodConfig::for_bevy();
        assert!(lod.validate().is_ok());

        // Should be in ascending order
        for window in lod.distance_thresholds.windows(2) {
            assert!(window[0] < window[1]);
        }
    }

    #[test]
    fn test_naming_for_bevy() {
        let naming = NamingConfig::for_bevy();
        let filename = naming.generate_filename("stone_pillar", "var_0001_12345", "glb");

        // Should be lowercase, no prefix
        assert_eq!(filename, "stone_pillar_var_0001_12345.glb");
    }

    #[test]
    fn test_material_metallic_roughness() {
        let material = MaterialConfig::for_bevy();
        assert!(material.generate_metallic_roughness);
        assert_eq!(material.system, MaterialSystem::Pbr);
    }

    #[test]
    fn test_export_format_capabilities() {
        assert!(ExportFormat::Gltf.supports_lod());
        assert!(ExportFormat::Gltf.supports_materials());
        assert_eq!(ExportFormat::Gltf.extension(), "glb");
    }
}
