use super::super::{Algorithm, Point};
use super::circle_base::CircleBase;

pub struct BaselineCircle {
    pub base: CircleBase,
    points: Vec<Point>,
}

impl Default for BaselineCircle {
    fn default() -> Self {
        let mut s = Self {
            base: CircleBase::new(0, 0, 15),
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

    fn category(&self) -> &'static str {
        "circle"
    }

    fn compute(&mut self) {
        self.points.clear();

        // Brute-force: iterate through a bounding square and check the implicit equation
        let r2 = self.base.radius * self.base.radius;

        for dy in -self.base.radius..=self.base.radius {
            for dx in -self.base.radius..=self.base.radius {
                // Check if this pixel is approximately on the circle
                if dx * dx + dy * dy <= r2 {
                    self.points.push(Point {
                        x: self.base.center_x + dx,
                        y: self.base.center_y + dy,
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
        self.base.draw_ui_base(ui)
    }

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        self.base.overlay_circle_base()
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        self.base.cell_info_base(cx, cy)
    }
}
