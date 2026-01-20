// Drawing tools for the canvas editor.

use crate::Canvas;
use egui::Color32;
use tracing::{debug, trace};

pub trait Tool {
    fn apply(&self, canvas: &mut Canvas, x: u32, y: u32);

    fn name(&self) -> &str;

    fn cursor_size(&self) -> u32 {
        1
    }
}

#[derive(Debug, Clone)]
pub struct Brush {
    pub size: u32,
    pub color: Color32,
}

impl Brush {
    pub fn new(size: u32, color: Color32) -> Self {
        Self { size, color }
    }
}

impl Tool for Brush {
    fn apply(&self, canvas: &mut Canvas, x: u32, y: u32) {
        trace!(
            "Applying Brush at ({}, {}) with size {} and color {:?}",
            x,
            y,
            self.size,
            self.color
        );
        let half_size = self.size as i32 / 2;
        for dy in -half_size..=half_size {
            for dx in -half_size..=half_size {
                let px = x as i32 + dx;
                let py = y as i32 + dy;
                if px >= 0 && py >= 0 {
                    canvas.set_pixel(px as u32, py as u32, self.color);
                }
            }
        }
        debug!("Brush applied {} pixels", self.size * self.size);
    }

    fn name(&self) -> &str {
        "Brush"
    }

    fn cursor_size(&self) -> u32 {
        self.size
    }
}

#[derive(Debug, Clone)]
pub struct Eraser {
    pub size: u32,
    pub erase_color: Color32,
}

impl Eraser {
    pub fn new(size: u32) -> Self {
        debug!("Creating Eraser tool with size {}", size);
        Self {
            size,
            erase_color: Color32::TRANSPARENT,
        }
    }

    pub fn with_color(size: u32, color: Color32) -> Self {
        debug!(
            "Creating Eraser tool with size {} and color {:?}",
            size, color
        );
        Self {
            size,
            erase_color: color,
        }
    }
}

impl Tool for Eraser {
    fn apply(&self, canvas: &mut Canvas, x: u32, y: u32) {
        trace!("Applying Eraser at ({}, {}) with size {}", x, y, self.size);

        let half_size = self.size as i32 / 2;
        for dy in -half_size..=half_size {
            for dx in -half_size..=half_size {
                let px = x as i32 + dx;
                let py = y as i32 + dy;
                if px >= 0 && py >= 0 {
                    canvas.set_pixel(px as u32, py as u32, self.erase_color);
                }
            }
        }
        debug!("Eraser applied {} pixels", self.size * self.size);
    }

    fn name(&self) -> &str {
        "Eraser"
    }

    fn cursor_size(&self) -> u32 {
        self.size
    }
}

#[derive(Debug, Clone)]
pub struct Fill {
    pub color: Color32,
}

impl Fill {
    pub fn new(color: Color32) -> Self {
        Self { color }
    }
}

impl Tool for Fill {
    fn apply(&self, canvas: &mut Canvas, x: u32, y: u32) {
        trace!(
            "Starting flood fill at ({}, {}) with color {:?}",
            x,
            y,
            self.color
        );

        // Get the target color (what we're replacing)
        let target_color = match canvas.get_pixel(x, y) {
            Some(color) => color,
            None => {
                debug!("Fill attempted at out-of-bounds position ({}, {})", x, y);
                return;
            }
        };

        // If target is already the fill color, nothing to do
        if target_color == self.color {
            debug!("Target color already matches fill color, skipping fill");
            return;
        }

        // Do the flood fill
        flood_fill_recursive(canvas, x, y, target_color, self.color);

        debug!("Flood fill completed");
    }

    fn name(&self) -> &str {
        "Fill"
    }
}

fn flood_fill_recursive(
    canvas: &mut Canvas,
    x: u32,
    y: u32,
    target: Color32,
    replacement: Color32,
) {
    // Check if this pixel needs filling
    let current = match canvas.get_pixel(x, y) {
        Some(color) => color,
        None => return, // Out of bounds
    };

    if current != target {
        return;
    }

    // Fill this pixel
    canvas.set_pixel(x, y, replacement);

    // Recursively fill neighboring pixels
    if x > 0 {
        flood_fill_recursive(canvas, x - 1, y, target, replacement);
    }
    if y > 0 {
        flood_fill_recursive(canvas, x, y - 1, target, replacement);
    }
    flood_fill_recursive(canvas, x + 1, y, target, replacement);
    flood_fill_recursive(canvas, x, y + 1, target, replacement);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    #[test]
    fn test_brush() {
        let mut canvas = Canvas::new(10, 10, Color32::WHITE);
        let brush = Brush {
            size: 3,
            color: Color32::BLACK,
        };

        brush.apply(&mut canvas, 5, 5);

        for y in 4..=6 {
            for x in 4..=6 {
                assert_eq!(canvas.get_pixel(x, y), Some(Color32::BLACK));
            }
        }
    }

    #[test]
    fn test_eraser_transparent() {
        let mut canvas = Canvas::new(10, 10, Color32::BLACK);
        let eraser = Eraser::new(3); // Defaults to transparent

        eraser.apply(&mut canvas, 5, 5);

        for y in 4..=6 {
            for x in 4..=6 {
                assert_eq!(canvas.get_pixel(x, y), Some(Color32::TRANSPARENT));
            }
        }
    }

    #[test]
    fn test_eraser_to_white() {
        let mut canvas = Canvas::new(10, 10, Color32::BLACK);
        let eraser = Eraser::with_color(3, Color32::WHITE); // Erase to white

        eraser.apply(&mut canvas, 5, 5);

        for y in 4..=6 {
            for x in 4..=6 {
                assert_eq!(canvas.get_pixel(x, y), Some(Color32::WHITE));
            }
        }
    }

    #[test]
    fn test_fill_entire_canvas() {
        let mut canvas = Canvas::new(5, 5, Color32::WHITE);
        let fill = Fill::new(Color32::RED);

        fill.apply(&mut canvas, 2, 2);

        // Entire canvas should be red
        for y in 0..5 {
            for x in 0..5 {
                assert_eq!(canvas.get_pixel(x, y), Some(Color32::RED));
            }
        }
    }

    #[test]
    fn test_fill_bounded_region() {
        let mut canvas = Canvas::new(10, 10, Color32::WHITE);

        // Draw a black border
        for i in 0..10 {
            canvas.set_pixel(i, 0, Color32::BLACK);
            canvas.set_pixel(i, 9, Color32::BLACK);
            canvas.set_pixel(0, i, Color32::BLACK);
            canvas.set_pixel(9, i, Color32::BLACK);
        }

        // Fill inside the border
        let fill = Fill::new(Color32::GREEN);
        fill.apply(&mut canvas, 5, 5);

        // Inside should be green, border should still be black
        assert_eq!(canvas.get_pixel(5, 5), Some(Color32::GREEN));
        assert_eq!(canvas.get_pixel(1, 1), Some(Color32::GREEN));
        assert_eq!(canvas.get_pixel(0, 0), Some(Color32::BLACK));
    }
}
