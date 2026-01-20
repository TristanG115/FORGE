//! Project management and style consistency for FORGE.
//!
//! Projects group related sessions and maintain consistent visual style across all assets.
//! This enables "same artist" consistency - all assets in a project share aesthetic properties.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use crate::{AssetClass, BaseInputRefV1, ParameterSetV1, Seed, SessionV1};

/// Visual texture style for assets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureStyle {
    /// Pixel art with specified pixel size (e.g., 16x16 pixels per unit)
    PixelArt { pixel_size: u32 },
    /// Photorealistic textures
    Realistic,
    /// Hand-painted artistic style
    HandPainted,
    /// Stylized/cartoon rendering
    Stylized,
    /// Low-poly/flat shading aesthetic
    LowPoly,
}

impl Default for TextureStyle {
    fn default() -> Self {
        TextureStyle::Stylized
    }
}

/// Aesthetic profile defining overall visual character.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AestheticProfile {
    /// Geometry complexity: 0.0 = simple/blocky, 1.0 = highly detailed
    pub geometry_complexity: f32,

    /// Realism level: 0.0 = cartoon/abstract, 1.0 = photorealistic
    pub realism: f32,

    /// Default wear/damage tendency: 0.0 = pristine, 1.0 = heavily weathered
    pub wear_tendency: f32,

    /// Symmetry preference: 0.0 = asymmetric, 1.0 = highly symmetric
    pub symmetry_preference: f32,
}

impl Default for AestheticProfile {
    fn default() -> Self {
        Self {
            geometry_complexity: 0.5,
            realism: 0.5,
            wear_tendency: 0.3,
            symmetry_preference: 0.5,
        }
    }
}

impl AestheticProfile {
    /// Validate aesthetic profile values are in valid ranges.
    pub fn validate(&self) -> Result<(), ProjectError> {
        let fields = [
            ("geometry_complexity", self.geometry_complexity),
            ("realism", self.realism),
            ("wear_tendency", self.wear_tendency),
            ("symmetry_preference", self.symmetry_preference),
        ];

        for (name, value) in fields {
            if !(0.0..=1.0).contains(&value) {
                tracing::error!(
                    field = name,
                    value = value,
                    "aesthetic profile value out of range [0.0, 1.0]"
                );
                return Err(ProjectError::InvalidAestheticValue {
                    field: name.to_string(),
                    value,
                });
            }
        }

        Ok(())
    }
}

/// Color palette definition for consistent coloring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorPalette {
    /// Palette name (e.g., "Minecraft", "Dark Fantasy")
    pub name: String,

    /// Primary colors in the palette (RGB, 0.0-1.0)
    pub colors: Vec<[f32; 3]>,

    /// Whether to strictly limit to these colors or use as guidance
    pub strict: bool,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            colors: vec![
                [0.5, 0.5, 0.5], // Gray
                [0.6, 0.4, 0.2], // Brown
                [0.3, 0.3, 0.3], // Dark gray
            ],
            strict: false,
        }
    }
}

impl ColorPalette {
    /// Create a Minecraft-style palette.
    pub fn minecraft() -> Self {
        Self {
            name: "Minecraft".into(),
            colors: vec![
                [0.545, 0.271, 0.075], // Brown (wood)
                [0.502, 0.502, 0.502], // Gray (stone)
                [0.133, 0.545, 0.133], // Green (grass)
                [0.824, 0.706, 0.549], // Tan (sand)
            ],
            strict: true,
        }
    }

    /// Create a dark fantasy palette.
    pub fn dark_fantasy() -> Self {
        Self {
            name: "Dark Fantasy".into(),
            colors: vec![
                [0.2, 0.2, 0.2],    // Dark gray
                [0.3, 0.25, 0.2],   // Dark brown
                [0.15, 0.15, 0.18], // Near black
                [0.4, 0.35, 0.3],   // Weathered stone
            ],
            strict: false,
        }
    }

    /// Validate all colors are in valid RGB range.
    pub fn validate(&self) -> Result<(), ProjectError> {
        for (i, color) in self.colors.iter().enumerate() {
            for (channel, &value) in color.iter().enumerate() {
                if !(0.0..=1.0).contains(&value) {
                    tracing::error!(
                        color_index = i,
                        channel = channel,
                        value = value,
                        "color value out of range"
                    );
                    return Err(ProjectError::InvalidColorValue {
                        color_index: i,
                        channel,
                        value,
                    });
                }
            }
        }

        if self.colors.is_empty() {
            tracing::warn!("color palette has no colors defined");
        }

        Ok(())
    }
}

/// Reference to a previously approved asset for style learning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetReference {
    /// Which approval this references
    pub approved_id: String,

    /// Path to the generated 3D asset (for visual reference)
    pub asset_path: Option<String>,

    /// Timestamp when this was approved
    pub approved_at: i64,
}

/// Project-wide style profile for consistency across all sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectStyleProfile {
    /// Visual texture style
    pub texture_style: TextureStyle,

    /// Overall aesthetic preferences
    pub aesthetic: AestheticProfile,

    /// Color palette for consistent coloring
    pub color_palette: ColorPalette,

    /// Pixel density: higher = more detailed, lower = more pixelated/blocky
    pub pixel_density: f32,

    /// Edge sharpness: 0.0 = smooth/beveled, 1.0 = hard edges
    pub edge_sharpness: f32,

    /// Previously approved assets to learn from (for future AI learning)
    pub reference_assets: Vec<AssetReference>,

    /// Style embeddings (future: AI-learned style vector)
    pub style_embeddings: Vec<f32>,

    /// Free-form notes about the desired style
    pub style_notes: String,
}

impl Default for ProjectStyleProfile {
    fn default() -> Self {
        Self {
            texture_style: TextureStyle::default(),
            aesthetic: AestheticProfile::default(),
            color_palette: ColorPalette::default(),
            pixel_density: 0.5,
            edge_sharpness: 0.5,
            reference_assets: Vec::new(),
            style_embeddings: Vec::new(),
            style_notes: String::new(),
        }
    }
}

impl ProjectStyleProfile {
    /// Create a Minecraft-style profile.
    pub fn minecraft() -> Self {
        Self {
            texture_style: TextureStyle::PixelArt { pixel_size: 16 },
            aesthetic: AestheticProfile {
                geometry_complexity: 0.2, // Very simple, blocky
                realism: 0.0,             // Not realistic at all
                wear_tendency: 0.2,       // Some wear but mostly clean
                symmetry_preference: 0.7, // Fairly symmetric
            },
            color_palette: ColorPalette::minecraft(),
            pixel_density: 0.1,  // Very low = blocky
            edge_sharpness: 1.0, // Hard edges, no smoothing
            reference_assets: Vec::new(),
            style_embeddings: Vec::new(),
            style_notes: "Blocky, pixelated aesthetic inspired by Minecraft".into(),
        }
    }

    /// Create a dark fantasy style profile.
    pub fn dark_fantasy() -> Self {
        Self {
            texture_style: TextureStyle::Realistic,
            aesthetic: AestheticProfile {
                geometry_complexity: 0.7, // Detailed
                realism: 0.8,             // Fairly realistic
                wear_tendency: 0.8,       // Heavily weathered
                symmetry_preference: 0.3, // More asymmetric
            },
            color_palette: ColorPalette::dark_fantasy(),
            pixel_density: 0.8,  // High detail
            edge_sharpness: 0.3, // Some beveling/wear
            reference_assets: Vec::new(),
            style_embeddings: Vec::new(),
            style_notes: "Dark, weathered, realistic medieval fantasy".into(),
        }
    }

    /// Apply this style profile to a parameter set, adjusting defaults.
    pub fn apply_to_params(&self, mut params: ParameterSetV1) -> ParameterSetV1 {
        tracing::debug!("applying style profile to parameters");

        // Apply aesthetic preferences to parameters
        params.erosion_intensity.value = self.aesthetic.wear_tendency;
        params.symmetry_break.value = 1.0 - self.aesthetic.symmetry_preference;
        params.detail_density.value = self.aesthetic.geometry_complexity;
        params.bevel_amount.value = 1.0 - self.edge_sharpness;

        // Clamp everything after application
        params = params.clamp_all();

        tracing::debug!(
            erosion = params.erosion_intensity.value,
            symmetry_break = params.symmetry_break.value,
            detail = params.detail_density.value,
            bevel = params.bevel_amount.value,
            "style profile applied to parameters"
        );

        params
    }

    /// Add a reference asset to learn from (called after approval).
    pub fn add_reference(&mut self, approved_id: String, asset_path: Option<String>) {
        tracing::info!(
            approved_id = %approved_id,
            has_path = asset_path.is_some(),
            "adding reference asset to style profile"
        );

        let reference = AssetReference {
            approved_id,
            asset_path,
            approved_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        self.reference_assets.push(reference);

        tracing::debug!(
            total_references = self.reference_assets.len(),
            "reference asset added"
        );
    }

    /// Validate the style profile.
    pub fn validate(&self) -> Result<(), ProjectError> {
        self.aesthetic.validate()?;
        self.color_palette.validate()?;

        if !(0.0..=1.0).contains(&self.pixel_density) {
            tracing::error!(
                pixel_density = self.pixel_density,
                "pixel_density out of range"
            );
            return Err(ProjectError::InvalidStyleValue {
                field: "pixel_density".into(),
                value: self.pixel_density,
            });
        }

        if !(0.0..=1.0).contains(&self.edge_sharpness) {
            tracing::error!(
                edge_sharpness = self.edge_sharpness,
                "edge_sharpness out of range"
            );
            return Err(ProjectError::InvalidStyleValue {
                field: "edge_sharpness".into(),
                value: self.edge_sharpness,
            });
        }

        Ok(())
    }
}

/// A project groups related sessions and maintains style consistency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub project_id: Uuid,
    pub name: String,
    pub description: String,
    pub style_profile: ProjectStyleProfile,

    /// Session IDs belonging to this project
    pub sessions: Vec<Uuid>,

    /// Per-asset-class parameter overrides (optional fine-tuning)
    pub class_overrides: HashMap<String, ParameterSetV1>,

    pub created_at: i64,
    pub last_modified: i64,
}

impl Project {
    /// Create a new project with a style profile.
    pub fn new(
        name: impl Into<String>,
        style_profile: ProjectStyleProfile,
    ) -> Result<Self, ProjectError> {
        let name = name.into();

        if name.trim().is_empty() {
            tracing::error!("project name cannot be empty");
            return Err(ProjectError::EmptyName);
        }

        style_profile.validate()?;

        let project_id = Uuid::new_v4();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        tracing::info!(
            project_id = %project_id,
            name = %name,
            "creating new project"
        );

        Ok(Self {
            project_id,
            name,
            description: String::new(),
            style_profile,
            sessions: Vec::new(),
            class_overrides: HashMap::new(),
            created_at: now,
            last_modified: now,
        })
    }

    /// Create a session within this project, inheriting style profile.
    pub fn create_session(
        &mut self,
        asset_class: AssetClass,
        base_input: BaseInputRefV1,
        base_seed: Seed,
    ) -> Result<SessionV1, ProjectError> {
        tracing::info!(
            project_id = %self.project_id,
            asset_class = ?asset_class,
            "creating session with project style"
        );

        // Create base session
        let mut session = SessionV1::new(asset_class.clone(), base_input, base_seed)
            .map_err(ProjectError::SessionCreation)?;

        // Apply project style profile to parameters
        session.base_params = self.style_profile.apply_to_params(session.base_params);

        // Apply asset-class-specific overrides if they exist
        if let Some(override_params) = self.class_overrides.get(&format!("{:?}", asset_class)) {
            tracing::debug!(
                asset_class = ?asset_class,
                "applying class-specific parameter overrides"
            );
            session.base_params = override_params.clone();
        }

        // Register session with project
        self.sessions.push(session.session_id);
        self.update_modified_time();

        tracing::info!(
            session_id = %session.session_id,
            total_sessions = self.sessions.len(),
            "session created and added to project"
        );

        Ok(session)
    }

    /// Add a reference asset to the style profile (called after approval).
    pub fn learn_from_approval(&mut self, approved_id: String, asset_path: Option<String>) {
        tracing::info!(
            project_id = %self.project_id,
            approved_id = %approved_id,
            "learning from approved asset"
        );

        self.style_profile.add_reference(approved_id, asset_path);
        self.update_modified_time();

        // Future: Extract style features and update embeddings here
        tracing::debug!("style learning placeholder - future AI integration point");
    }

    /// Set asset-class-specific parameter overrides.
    pub fn set_class_override(&mut self, asset_class: AssetClass, params: ParameterSetV1) {
        tracing::info!(
            project_id = %self.project_id,
            asset_class = ?asset_class,
            "setting class-specific parameter override"
        );

        let key = format!("{:?}", asset_class);
        self.class_overrides.insert(key, params);
        self.update_modified_time();
    }

    /// Remove asset-class-specific overrides.
    pub fn clear_class_override(&mut self, asset_class: &AssetClass) {
        let key = format!("{:?}", asset_class);
        if self.class_overrides.remove(&key).is_some() {
            tracing::info!(
                project_id = %self.project_id,
                asset_class = ?asset_class,
                "cleared class-specific parameter override"
            );
            self.update_modified_time();
        }
    }

    /// Update last modified timestamp.
    fn update_modified_time(&mut self) {
        self.last_modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
    }

    /// Validate project data.
    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.name.trim().is_empty() {
            return Err(ProjectError::EmptyName);
        }

        self.style_profile.validate()?;

        for params in self.class_overrides.values() {
            params
                .validate()
                .map_err(|e| ProjectError::InvalidOverrideParams(e))?;
        }

        Ok(())
    }
}

/// Project-related errors.
#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("project name cannot be empty")]
    EmptyName,

    #[error("invalid aesthetic value '{field}': {value} (must be in [0.0, 1.0])")]
    InvalidAestheticValue { field: String, value: f32 },

    #[error("invalid style value '{field}': {value} (must be in [0.0, 1.0])")]
    InvalidStyleValue { field: String, value: f32 },

    #[error("invalid color value at index {color_index}, channel {channel}: {value}")]
    InvalidColorValue {
        color_index: usize,
        channel: usize,
        value: f32,
    },

    #[error("session creation failed: {0}")]
    SessionCreation(#[from] crate::SessionError),

    #[error("invalid override parameters: {0}")]
    InvalidOverrideParams(#[from] crate::ParamError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aesthetic_profile_validation() {
        let valid = AestheticProfile::default();
        assert!(valid.validate().is_ok());

        let invalid = AestheticProfile {
            geometry_complexity: 1.5,
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_color_palette_validation() {
        let valid = ColorPalette::default();
        assert!(valid.validate().is_ok());

        let invalid = ColorPalette {
            name: "Bad".into(),
            colors: vec![[1.0, 2.0, 0.5]], // Out of range
            strict: false,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_minecraft_preset() {
        let style = ProjectStyleProfile::minecraft();
        assert!(style.validate().is_ok());
        assert_eq!(style.pixel_density, 0.1);
        assert_eq!(style.edge_sharpness, 1.0);
    }

    #[test]
    fn test_dark_fantasy_preset() {
        let style = ProjectStyleProfile::dark_fantasy();
        assert!(style.validate().is_ok());
        assert_eq!(style.aesthetic.wear_tendency, 0.8);
    }

    #[test]
    fn test_style_application_to_params() {
        let style = ProjectStyleProfile::minecraft();
        let params = ParameterSetV1::default();

        let styled_params = style.apply_to_params(params);

        // Should have low erosion (clean blocks)
        assert!(styled_params.erosion_intensity.value < 0.3);
        // Should have low symmetry break (symmetric)
        assert!(styled_params.symmetry_break.value < 0.4);
    }

    #[test]
    fn test_project_creation() {
        let style = ProjectStyleProfile::default();
        let project = Project::new("Test Project", style);

        assert!(project.is_ok());
        let project = project.unwrap();
        assert_eq!(project.sessions.len(), 0);
    }

    #[test]
    fn test_project_empty_name_rejected() {
        let style = ProjectStyleProfile::default();
        let project = Project::new("", style);

        assert!(project.is_err());
    }

    #[test]
    fn test_add_reference_asset() {
        let mut style = ProjectStyleProfile::default();
        assert_eq!(style.reference_assets.len(), 0);

        style.add_reference("appr_001".into(), Some("/path/to/asset.fbx".into()));
        assert_eq!(style.reference_assets.len(), 1);
        assert_eq!(style.reference_assets[0].approved_id, "appr_001");
    }
}
