use super::{Algorithm, Point};
use rayon::prelude::*;

pub struct ParallelMidpointCircle {
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    pub show_smooth_overlay: bool,
    points: Vec<Point>,
}

impl Default for ParallelMidpointCircle {
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

impl Algorithm for ParallelMidpointCircle {
    fn name(&self) -> &str {
        "Parallel Midpoint Circle"
    }

    fn compute(&mut self) {
        self.points.clear();

        // Phase 1: Generate edge points sequentially (the midpoint algorithm inherently requires this)
        let mut edge_points = Vec::new();
        let mut x = 0i32;
        let mut y = self.radius;
        let mut d = 1 - self.radius;

        while x <= y {
            edge_points.push((x, y));
            if d < 0 {
                d += 2 * x + 3;
            } else {
                d += 2 * (x - y) + 5;
                y -= 1;
            }
            x += 1;
        }

        // Phase 2: Parallelize the octant reflection and collection using Rayon
        let all_octant_points: Vec<Point> = edge_points
            .par_iter()
            .flat_map(|&(x, y)| self.generate_octants(x, y))
            .collect();

        self.points = all_octant_points;
    }

    fn points(&self) -> &[Point] {
        &self.points
    }

    fn color(&self) -> [f32; 3] {
        [0.0, 1.0, 0.5] // Green-cyan to distinguish from sequential version
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

impl ParallelMidpointCircle {
    /// Generate all 8 octant reflections for a single edge point.
    /// This is parallelized by Rayon at the per-edge-point level.
    fn generate_octants(&self, x: i32, y: i32) -> Vec<Point> {
        let cx = self.center_x;
        let cy = self.center_y;

        vec![
            Point {
                x: cx + x,
                y: cy + y,
            },
            Point {
                x: cx - x,
                y: cy + y,
            },
            Point {
                x: cx + x,
                y: cy - y,
            },
            Point {
                x: cx - x,
                y: cy - y,
            },
            Point {
                x: cx + y,
                y: cy + x,
            },
            Point {
                x: cx - y,
                y: cy + x,
            },
            Point {
                x: cx + y,
                y: cy - x,
            },
            Point {
                x: cx - y,
                y: cy - x,
            },
        ]
    }

    /// Implicit circle equation: negative ⟹ inside, zero ⟹ on, positive ⟹ outside.
    pub fn decision_parameter(&self, x: i32, y: i32) -> i32 {
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        dx * dx + dy * dy - self.radius * self.radius
    }
}
