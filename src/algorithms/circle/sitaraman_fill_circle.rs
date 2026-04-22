use super::super::{Algorithm, Point};
use super::circle_base::{CircleBase, PALETTE};

pub struct SitaramanFillCircle {
    pub base: CircleBase,
    /// Which concentric ring each point belongs to (1 … radius).
    point_radii: Vec<i32>,
    /// Starting index of each ring in the points array (0-based).
    ring_start_indices: Vec<usize>,
}

impl Default for SitaramanFillCircle {
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

impl Algorithm for SitaramanFillCircle {
    fn name(&self) -> &str {
        "Sitaraman Fill Circle"
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

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.base.draw_ui_with_duplicates(ui)
    }

    fn category(&self) -> &'static str {
        "circle"
    }

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        self.base.overlay_circle_base()
    }
}

impl SitaramanFillCircle {
    fn compute_ring(&mut self, r: i32) {
        let mut x = 0i32;
        let mut y = r;
        let mut d = 1 - r;

        while x <= y {
            self.plot_octants(x, y, r);

            if d < 0 {
                d += 2 * x + 3;
            } else {
                // Check fill decision parameter d' = d + 2y − 2r − 1
                if d + 2 * y - 2 * r - 1 < 0 {
                    self.plot_octants(x + 1, y, r);
                }
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
