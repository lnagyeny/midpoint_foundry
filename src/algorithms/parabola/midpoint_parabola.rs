use crate::algorithms::{Algorithm, Point};

pub struct MidpointParabola {
    pub center_x: i32,
    pub center_y: i32,
    pub k: f32,
    pub show_smooth_overlay: bool,
    points: Vec<Point>,
}

impl Default for MidpointParabola {
    fn default() -> Self {
        let mut s = Self {
            center_x: 0,
            center_y: 0,
            k: 0.1,
            show_smooth_overlay: false,
            points: Vec::new(),
        };
        s.compute();
        s
    }
}

impl Algorithm for MidpointParabola {
    fn name(&self) -> &str {
        "Midpoint Parabola"
    }

    fn compute(&mut self) {
        self.points.clear();
        let (cx, cy) = (self.center_x, self.center_y);
        let k = self.k;

        let mut x = 0i32;
        let mut y = 0i32;

        // Region 1: |slope| = 2kx < 1  →  step in x, decide y
        // Midpoint: (x+1, y+0.5),  d = k*(x+1)² - (y+0.5)
        // Initial: k*1² - 0.5
        let mut d = k - 0.5_f32;

        while 2.0 * k * (x as f32) < 1.0 && x <= 100 && y <= 100 {
            self.plot_symmetric(cx, cy, x, y);
            if d >= 0.0 {
                // midpoint below curve → choose (x+1, y+1)
                y += 1;
                d += k * (2.0 * x as f32 + 3.0) - 1.0;
            } else {
                // midpoint above curve → choose (x+1, y)
                d += k * (2.0 * x as f32 + 3.0);
            }
            x += 1;
        }

        // Region 2: |slope| >= 1  →  step in y, decide x
        // Midpoint: (x+0.5, y+1),  d = k*(x+0.5)² - (y+1)
        let mut d2 = k * (x as f32 + 0.5) * (x as f32 + 0.5) - (y + 1) as f32;

        while x <= 100 && y <= 100 {
            self.plot_symmetric(cx, cy, x, y);
            if d2 < 0.0 {
                // midpoint left of curve → choose (x+1, y+1)
                x += 1;
                d2 += 2.0 * k * x as f32 - 1.0; // x already incremented: 2k*(old_x+1) - 1
            } else {
                // midpoint right of/on curve → choose (x, y+1)
                d2 -= 1.0;
            }
            y += 1; // y always increments → loop always terminates
        }
    }
    fn points(&self) -> &[Point] {
        &self.points
    }

    fn color(&self) -> [f32; 3] {
        [0.0, 0.78, 1.0]
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
            .add(egui::Slider::new(&mut self.k, 0.01f32..=1.0).text("Width factor k"))
            .changed();

        ui.checkbox(
            &mut self.show_smooth_overlay,
            "Draw analytic parabola overlay",
        );

        changed
    }

    fn overlay_parabola(&self) -> Option<(i32, i32, i32)> {
        if self.show_smooth_overlay {
            Some((self.center_x, self.center_y, self.k as i32))
        } else {
            None
        }
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        let d = self.decision_parameter(cx, cy);
        Some(format!(
            "x²−y/k = {d}\n({}  parabola)",
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
        "parabola"
    }
}

impl MidpointParabola {
    fn plot_symmetric(&mut self, cx: i32, cy: i32, x: i32, y: i32) {
        self.points.push(Point {
            x: cx + x,
            y: cy + y,
        });

        self.points.push(Point {
            x: cx - x,
            y: cy + y,
        });
    }

    pub fn decision_parameter(&self, x: i32, y: i32) -> i32 {
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        let inner = (dy as f32 / self.k) as i64;
        ((dx * dx) as i64 - inner) as i32
    }
}
