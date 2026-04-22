use super::super::{Algorithm, Point};
use super::circle_base::CircleBase;
use rayon::prelude::*;

pub struct ParallelMidpointCircle {
    pub base: CircleBase,
    points: Vec<Point>,
}

impl Default for ParallelMidpointCircle {
    fn default() -> Self {
        let mut s = Self {
            base: CircleBase::new(0, 0, 15),
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
        let mut y = self.base.radius;
        let mut d = 1 - self.base.radius;

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
        self.base.draw_ui_base(ui)
    }

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        self.base.overlay_circle_base()
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        self.base.cell_info_base(cx, cy)
    }

    fn category(&self) -> &'static str {
        "circle"
    }
}

impl ParallelMidpointCircle {
    /// Generate all 8 octant reflections for a single edge point.
    /// This is parallelized by Rayon at the per-edge-point level.
    fn generate_octants(&self, x: i32, y: i32) -> Vec<Point> {
        let cx = self.base.center_x;
        let cy = self.base.center_y;

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
}
