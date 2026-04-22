use super::super::{Algorithm, Point};
use super::circle_base::CircleBase;
use std::collections::HashSet;

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

const PALETTE: [[f32; 3]; 7] = [
    [0.00, 0.45, 0.70], // deep blue
    [0.55, 0.35, 0.85], // purple
    [0.30, 0.70, 0.20], // green
    [0.80, 0.75, 0.00], // yellow-olive
    [0.00, 0.60, 0.50], // teal
    [0.90, 0.50, 0.00], // orange (not too redish)
    [0.40, 0.40, 0.40], // neutral gray (good for contrast reference);
];

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
        if !self.base.show_duplicates {
            let r = *self.point_radii.get(i)? as usize;
            return Some(PALETTE[r % 3]);
        }
        // Lazy init for duplicates
        if self.base.duplicate_cache.read().unwrap().is_none() {
            let mut seen = HashSet::new();
            let mut dupes = HashSet::new();
            for p in &self.base.points {
                if !seen.insert((p.x, p.y)) {
                    dupes.insert((p.x, p.y));
                }
            }
            *self.base.duplicate_cache.write().unwrap() = Some(dupes);
        }

        let cache = self.base.duplicate_cache.read().unwrap();
        let dupes = cache.as_ref().unwrap();
        let p = &self.base.points[i];
        if dupes.contains(&(p.x, p.y)) {
            Some([1.0, 0.0, 0.0])
        } else {
            let r = *self.point_radii.get(i)? as usize;
            Some(PALETTE[r % 3])
        }
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = self.base.draw_ui_base(ui);

        changed |= ui
            .add(egui::Checkbox::new(
                &mut self.base.show_duplicates,
                "Show Duplicates",
            ))
            .changed();

        if changed {
            *self.base.duplicate_cache.write().unwrap() = None; // cache törlése
        }
        changed
    }

    fn category(&self) -> &'static str {
        "circle"
    }

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        Some((self.base.center_x, self.base.center_y, self.base.radius))
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
