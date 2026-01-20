// This is a canvas editor for FORGE UI
// It allows users to create their own templates or edit the creations from the AI models

use egui::Color32;
use tracing::{debug, info, trace, warn};

#[derive(Debug, Clone)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color32>,
}

impl Canvas {
    pub fn new(width: u32, height: u32, background: Color32) -> Self {
        info!(
            "Creating new canvas of size {}x{} with background color {:?}",
            width, height, background
        );

        let total_pixels = (width * height) as usize;
        debug!("Total pixels to initialize: {}", total_pixels);

        let pixels = vec![background; total_pixels];
        trace!("Canvas created");

        Self {
            width,
            height,
            pixels,
        }
    }

    // Check if coordinates are within canvas bounds
    fn is_valid_coordinate(&self, x: u32, y: u32) -> bool {
        debug!(
            "Checking coordinates ({}, {}) against canvas size {}x{}",
            x, y, self.width, self.height
        );
        let valid = x < self.width && y < self.height;

        debug!("Coordinate validity: {}", valid);
        valid;
    }

    //convert 2d coordinates to 1d index
    fn coord_to_index(&self, x: u32, y: u32) -> usize {
        let index = (y * self.width + x) as usize;
        debug!("Converted coordinates ({}, {}) to index {}", x, y, index);
        index;
    }

    // Get the color of a pixel at (x, y)
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Color32> {
        if !self.is_valid_coordinate(x, y) {
            warn!(
                "Requested pixel color at invalid coordinates ({}, {})",
                x, y
            );
            return None;
        }

        let index = self.coord_to_index(x, y);
        trace!("Getting pixel color at ({}, {}) with index {}", x, y, index);

        Some(self.pixels[index])
    }

    // Set the color of a pixel at (x, y)
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color32) -> bool {
        if !self.is_valid_coordinate(x, y) {
            warn!(
                "Attempted to set pixel color at invalid coordinates ({}, {})",
                x, y
            );
            return false;
        }

        let index = self.coord_to_index(x, y);
        trace!(
            "Setting pixel color at ({}, {}) with index {} to {:?}",
            x,
            y,
            index,
            color
        );

        self.pixels[index] = color;
        true
    }

    // Fill entire canvas with a color
    pub fn fill(&mut self, color: Color32) {
        info!("Filling canvas {:?}", color);
        for pixel in self.pixels.iter_mut() {
            *pixel = color;
        }
        trace!("Canvas fill complete");
    }

    // Clear canvas
    pub fn clear(&mut self) {
        info!("Clearing canvas");
        self.fill(Color32::WHITE);
        trace!("Canvas cleared");
    }

    // Get canvas dimensions
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

// Create a default canvas
impl Default for Canvas {
    fn default() -> Self {
        Canvas::new(512, 512, Color32::WHITE)
    }
}
