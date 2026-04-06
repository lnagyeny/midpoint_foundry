use super::{Algorithm, Point};

pub struct BaselineCircle {
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    pub show_smooth_overlay: bool,
    points: Vec<Point>,
}

impl Default for BaselineCircle {
    fn default() -> Self {
        let mut s = Self {
            center_x: 0,
            center_y: 0,
            radius: 15,
            show_smooth_overlay: false,
            points: Vec::new(),
        };
        s.compute();
        s
    }
}

impl Algorithm for BaselineCircle {
    fn name(&self) -> &str {
        "Baseline Circle (Implicit Equation)"
    }

    fn compute(&mut self) {
        self.points.clear();

        // Brute-force: iterate through a bounding square and check the implicit equation
        let r2 = self.radius * self.radius;

        for dy in -self.radius..=self.radius {
            for dx in -self.radius..=self.radius {
                // Check if this pixel is approximately on the circle
                if dx * dx + dy * dy <= r2 {
                    self.points.push(Point {
                        x: self.center_x + dx,
                        y: self.center_y + dy,
                    });
                }
            }
        }
    }

    fn points(&self) -> &[Point] {
        &self.points
    }

    fn color(&self) -> [f32; 3] {
        [1.0, 0.5, 0.0] // Orange to distinguish from other algorithms
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
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

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        if self.show_smooth_overlay {
            Some((self.center_x, self.center_y, self.radius))
        } else {
            None
        }
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
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
}

impl BaselineCircle {
    /// Implicit circle equation: negative ⟹ inside, zero ⟹ on, positive ⟹ outside.
    pub fn decision_parameter(&self, x: i32, y: i32) -> i32 {
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        dx * dx + dy * dy - self.radius * self.radius
    }
}
