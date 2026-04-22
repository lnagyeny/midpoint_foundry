use super::super::{Algorithm, Point};
use super::circle_base::{CircleBase, PALETTE};

pub struct GapFillCircle {
    pub base: CircleBase,
    /// Which concentric ring each point belongs to (1 … radius).
    point_radii: Vec<i32>,
    /// Starting index of each ring in the points array (0-based).
    ring_start_indices: Vec<usize>,
}

impl Default for GapFillCircle {
    fn default() -> Self {
        let mut s = Self {
            base: CircleBase::new(0, 0, 15),
            point_radii: Vec::new(),
            ring_start_indices: Vec::new(),
        };
        s.compute();
        s
    }
}

impl Algorithm for GapFillCircle {
    fn name(&self) -> &str {
        "Gap-Fill Circle"
    }

    fn compute(&mut self) {
        self.base.points.clear();
        self.point_radii.clear();
        self.ring_start_indices.clear();
        for r in 1..=self.base.radius {
            self.ring_start_indices.push(self.base.points.len());
            self.compute_ring(r);
        }
    }

    fn points(&self) -> &[Point] {
        &self.base.points
    }

    fn color(&self) -> [f32; 3] {
        [0.0, 0.5, 1.0]
    }

    fn point_color_override(&self, i: usize) -> Option<[f32; 3]> {
        let ring_color = {
            let r = *self.point_radii.get(i)? as usize;
            Some(PALETTE[r % PALETTE.len()])
        };
        self.base.duplicate_color(i, ring_color)
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        for (i, p) in self.base.points.iter().enumerate() {
            if p.x == cx && p.y == cy {
                let _r = self.point_radii[i] as usize;
                // Find which ring this point belongs to
                let ring_index = (1..self.ring_start_indices.len())
                    .find(|&idx| self.ring_start_indices[idx] > i)
                    .unwrap_or(self.ring_start_indices.len())
                    - 1;

                let step_in_ring = i - self.ring_start_indices[ring_index] + 1;
                let ring_number = ring_index + 1; // Rings are 1-indexed

                return Some(format!("Step {} (ring {})", step_in_ring, ring_number));
            }
        }
        None
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.base.draw_ui_with_duplicates(ui)
    }

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        self.base.overlay_circle_base()
    }

    fn category(&self) -> &'static str {
        "circle"
    }
}

impl GapFillCircle {
    fn compute_ring(&mut self, r: i32) {
        let mut x = 0i32;
        let mut y = r;
        let mut d = 1 - r;

        while x <= y {
            self.plot_octants(x, y, r);

            if d < 0 {
                d += 2 * x + 3;
            } else {
                d += 2 * (x - y) + 5;
                y -= 1;
            }
            x += 1;
        }
    }

    fn plot_octants(&mut self, x: i32, y: i32, r: i32) {
        let (cx, cy) = (self.base.center_x, self.base.center_y);
        for &(dx, dy) in &[
            (x, y),
            (-x, y),
            (x, -y),
            (-x, -y),
            (y, x),
            (-y, x),
            (y, -x),
            (-y, -x),
        ] {
            self.base.points.push(Point {
                x: cx + dx,
                y: cy + dy,
            });
            self.point_radii.push(r);
        }
    }
}
