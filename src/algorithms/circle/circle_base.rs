use crate::algorithms::Point;
use egui::Ui;
use std::collections::HashSet;
use std::sync::RwLock;

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
}
