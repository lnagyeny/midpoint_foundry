use crate::algorithms::Point;
use egui::Ui;
use std::collections::HashSet;
use std::sync::RwLock;

pub const PALETTE: [[f32; 3]; 7] = [
    [0.00, 0.45, 0.70], // deep blue
    [0.55, 0.35, 0.85], // purple
    [0.30, 0.70, 0.20], // green
    [0.80, 0.75, 0.00], // yellow-olive
    [0.00, 0.60, 0.50], // teal
    [0.90, 0.50, 0.00], // orange
    [0.40, 0.40, 0.40], // neutral gray
];

/// Base struct containing common functionality for circle algorithms
pub struct CircleBase {
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    pub show_smooth_overlay: bool,
    pub points: Vec<Point>,
    pub show_duplicates: bool,
    pub duplicate_cache: RwLock<Option<HashSet<(i32, i32)>>>,
}

impl CircleBase {
    /// Create a new CircleBase with default values
    pub fn new(center_x: i32, center_y: i32, radius: i32) -> Self {
        Self {
            center_x,
            center_y,
            radius,
            show_smooth_overlay: false,
            points: Vec::new(),
            show_duplicates: false,
            duplicate_cache: RwLock::new(None),
        }
    }

    /// Common draw_ui implementation for circle parameters
    pub fn draw_ui_base(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.label("Center");
        ui.horizontal(|ui| {
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.center_x)
                        .prefix("X: ")
                        .range(-100..=100),
                )
                .changed();
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.center_y)
                        .prefix("Y: ")
                        .range(-100..=100),
                )
                .changed();
        });

        changed |= ui
            .add(egui::Slider::new(&mut self.radius, 1..=100).text("Radius"))
            .changed();

        ui.checkbox(
            &mut self.show_smooth_overlay,
            "Draw analytic circle overlay",
        );

        changed
    }

    /// draw_ui_base + "Show Duplicates" checkbox, cache invalidálással
    pub fn draw_ui_with_duplicates(&mut self, ui: &mut Ui) -> bool {
        let mut changed = self.draw_ui_base(ui);
        changed |= ui
            .add(egui::Checkbox::new(
                &mut self.show_duplicates,
                "Highlight duplicates",
            ))
            .changed();
        if changed {
            *self.duplicate_cache.write().unwrap() = None;
        }
        changed
    }

    /// Common overlay_circle implementation
    pub fn overlay_circle_base(&self) -> Option<(i32, i32, i32)> {
        if self.show_smooth_overlay {
            Some((self.center_x, self.center_y, self.radius))
        } else {
            None
        }
    }

    /// Common cell_info implementation
    pub fn cell_info_base(&self, cx: i32, cy: i32) -> Option<String> {
        let d = self.decision_parameter(cx, cy);
        Some(format!(
            "x²+y²−r²  =  {d}\n({}  circle)",
            if d < 0 {
                "inside"
            } else if d == 0 {
                "on"
            } else {
                "outside"
            }
        ))
    }

    /// Implicit circle equation: negative ⟹ inside, zero ⟹ on, positive ⟹ outside.
    pub fn decision_parameter(&self, x: i32, y: i32) -> i32 {
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        dx * dx + dy * dy - self.radius * self.radius
    }

    /// Check if the point at index `i` is a duplicate and return a highlight color if so
    /// uses 'fallback' if duplicates are disabled or not a duplicate
    pub fn duplicate_color(&self, i: usize, fallback: Option<[f32; 3]>) -> Option<[f32; 3]> {
        if !self.show_duplicates {
            return fallback;
        }
        if self.duplicate_cache.read().unwrap().is_none() {
            let mut seen = HashSet::new();
            let mut dupes = HashSet::new();
            for p in &self.points {
                if !seen.insert((p.x, p.y)) {
                    dupes.insert((p.x, p.y));
                }
            }
            *self.duplicate_cache.write().unwrap() = Some(dupes);
        }
        let cache = self.duplicate_cache.read().unwrap();
        let dupes = cache.as_ref().unwrap();
        let p = &self.points[i];
        if dupes.contains(&(p.x, p.y)) {
            Some([1.0, 0.0, 0.0])
        } else {
            fallback
        }
    }
}
