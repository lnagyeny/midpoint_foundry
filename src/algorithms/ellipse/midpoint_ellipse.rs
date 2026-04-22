use crate::algorithms::{Algorithm, Point};

pub struct MidpointEllipse {
    pub center_x: i32,
    pub center_y: i32,
    pub radius_x: i32,
    pub radius_y: i32,
    pub show_smooth_overlay: bool,
    points: Vec<Point>,
}

impl Default for MidpointEllipse {
    fn default() -> Self {
        let mut s = Self {
            center_x: 0,
            center_y: 0,
            radius_x: 30,
            radius_y: 20,
            show_smooth_overlay: false,
            points: Vec::new(),
        };
        s.compute();
        s
    }
}

impl Algorithm for MidpointEllipse {
    fn name(&self) -> &str {
        "Midpoint Ellipse"
    }

    fn compute(&mut self) {
        self.points.clear();
        let (cx, cy) = (self.center_x, self.center_y);

        let mut x = 0;
        let mut y = self.radius_y;

        let mut d1 = (self.radius_y * self.radius_y - self.radius_x * self.radius_x * self.radius_y)
            as f64
            + 0.25 * (self.radius_x * self.radius_x) as f64;
        let mut dx = 2 * self.radius_y * self.radius_y * x;
        let mut dy = 2 * self.radius_x * self.radius_x * y;

        // Region 1
        while dx < dy {
            self.plot_symmetric(cx, cy, x, y);
            if d1 < 0.0 {
                x += 1;
                dx += 2 * self.radius_y * self.radius_y;
                d1 += dx as f64 + (self.radius_y * self.radius_y) as f64;
            } else {
                x += 1;
                y -= 1;
                dx += 2 * self.radius_y * self.radius_y;
                dy -= 2 * self.radius_x * self.radius_x;
                d1 += (dx - dy + self.radius_y * self.radius_y) as f64;
            }
        }

        let mut d2 = (self.radius_y * self.radius_y) as f64 * (x as f64 + 0.5) * (x as f64 + 0.5)
            + (self.radius_x * self.radius_x) as f64 * (y - 1) as f64 * (y - 1) as f64
            - (self.radius_x * self.radius_x) as f64 * (self.radius_y * self.radius_y) as f64;

        // Region 2
        while y >= 0 {
            self.plot_symmetric(cx, cy, x, y);
            if d2 > 0.0 {
                y -= 1;
                dy -= 2 * self.radius_x * self.radius_x;
                d2 += (self.radius_x * self.radius_x) as f64 - dy as f64;
            } else {
                y -= 1;
                x += 1;
                dx += 2 * self.radius_y * self.radius_y;
                dy -= 2 * self.radius_x * self.radius_x;
                d2 += (dx - dy + self.radius_x * self.radius_x) as f64;
            }
        }
    }

    fn points(&self) -> &[Point] {
        &self.points
    }

    fn color(&self) -> [f32; 3] {
        [0.2, 0.2, 0.38]
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

        ui.label("Radii");
        ui.horizontal(|ui| {
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.radius_x)
                        .prefix("X: ")
                        .range(1..=100),
                )
                .changed();
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.radius_y)
                        .prefix("Y: ")
                        .range(1..=100),
                )
                .changed();
        });

        ui.checkbox(
            &mut self.show_smooth_overlay,
            "Draw analytic ellipse overlay",
        );

        changed
    }

    fn overlay_ellipse(&self) -> Option<(i32, i32, i32, i32)> {
        if self.show_smooth_overlay {
            Some((self.center_x, self.center_y, self.radius_x, self.radius_y))
        } else {
            None
        }
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        let d = self.decision_parameter(cx, cy);
        Some(format!(
            "b²x²+a²y²−a²b² = {d}\n({}  ellipse)",
            if d < 0 {
                "inside"
            } else if d == 0 {
                "on"
            } else {
                "outside"
            }
        ))
    }

    fn category(&self) -> &'static str {
        "ellipse"
    }
}

impl MidpointEllipse {
    fn plot_symmetric(&mut self, cx: i32, cy: i32, x: i32, y: i32) {
        self.points.push(Point {
            x: cx + x,
            y: cy + y,
        });
        self.points.push(Point {
            x: cx + x,
            y: cy - y,
        });
        self.points.push(Point {
            x: cx - x,
            y: cy + y,
        });
        self.points.push(Point {
            x: cx - x,
            y: cy - y,
        });
    }

    pub fn decision_parameter(&self, x: i32, y: i32) -> i64 {
        let dx = (x - self.center_x) as i64;
        let dy = (y - self.center_y) as i64;
        let a = self.radius_x as i64;
        let b = self.radius_y as i64;
        b * b * dx * dx + a * a * dy * dy - a * a * b * b
    }
}
