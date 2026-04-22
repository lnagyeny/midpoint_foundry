use super::{Algorithm, Point};
use std::collections::HashSet;
use std::sync::RwLock;

pub struct GapFillCircle {
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    points: Vec<Point>,
    /// Which concentric ring each point belongs to (1 … radius).
    point_radii: Vec<i32>,
    /// Starting index of each ring in the points array (0-based).
    ring_start_indices: Vec<usize>,
    pub show_duplicates: bool,
    duplicate_cache: RwLock<Option<HashSet<(i32, i32)>>>,
}

impl Default for GapFillCircle {
    fn default() -> Self {
        let mut s = Self {
            center_x: 0,
            center_y: 0,
            radius: 15,
            points: Vec::new(),
            point_radii: Vec::new(),
            ring_start_indices: Vec::new(),
            show_duplicates: false,
            duplicate_cache: RwLock::new(None),
        };
        s.compute();
        s
    }
}

// 12-colour vibrant palette, indexed by `ring_radius % 12`.
//const PALETTE: [[f32; 3]; 12] = [
//    [1.00, 0.00, 0.00], // red
//    [0.00, 1.00, 0.00], // green
//    [0.00, 0.00, 1.00], // blue
//    [1.00, 1.00, 0.00], // yellow
//    [1.00, 0.00, 1.00], // magenta
//    [0.00, 1.00, 1.00], // cyan
//    [1.00, 0.50, 0.00], // orange
//    [0.50, 0.00, 1.00], // purple
//    [0.00, 1.00, 0.50], // spring green
//    [1.00, 0.00, 0.50], // rose
//    [0.50, 1.00, 0.00], // lime
//    [0.00, 0.50, 1.00], // sky blue
//];

//const PALETTE: [[f32; 3]; 6] = [
//    [0.85, 0.57, 0.00], // harvest gold
//    [0.45, 0.75, 0.85], // glacier
//    [0.33, 1.00, 0.33], // alien green
//    [0.80, 0.72, 0.60], // dark beige
//    [0.20, 0.60, 0.40], // summer green
//    [0.85, 0.75, 0.85], // thistle;
//];

const PALETTE: [[f32; 3]; 7] = [
    [0.00, 0.45, 0.70], // deep blue
    [0.55, 0.35, 0.85], // purple
    [0.30, 0.70, 0.20], // green
    [0.80, 0.75, 0.00], // yellow-olive
    [0.00, 0.60, 0.50], // teal
    [0.90, 0.50, 0.00], // orange (not too redish)
    [0.40, 0.40, 0.40], // neutral gray (good for contrast reference);
];

impl Algorithm for GapFillCircle {
    fn name(&self) -> &str {
        "Gap-Fill Circle"
    }

    fn compute(&mut self) {
        self.points.clear();
        self.point_radii.clear();
        self.ring_start_indices.clear();
        for r in 1..=self.radius {
            self.ring_start_indices.push(self.points.len());
            self.compute_ring(r);
        }
    }

    fn points(&self) -> &[Point] {
        &self.points
    }

    fn color(&self) -> [f32; 3] {
        [0.0, 0.5, 1.0]
    }

    fn point_color_override(&self, i: usize) -> Option<[f32; 3]> {
        if !self.show_duplicates {
            let r = *self.point_radii.get(i)? as usize;
            return Some(PALETTE[r % 3]);
        }
        // Lazy init for duplicates
        if self.duplicate_cache.read().unwrap().is_none() {
            let mut seen = HashSet::new();
            let mut dupes = HashSet::new();
            for p in &self.points {
                if !seen.insert((p.x, p.y)) {
                    dupes.insert((p.x, p.y));
                }
            }
            *self.duplicate_cache.write().unwrap() = Some(dupes);
        }

        let cache = self.duplicate_cache.read().unwrap();
        let dupes = cache.as_ref().unwrap();
        let p = &self.points[i];
        if dupes.contains(&(p.x, p.y)) {
            Some([1.0, 0.0, 0.0])
        } else {
            let r = *self.point_radii.get(i)? as usize;
            Some(PALETTE[r % 3])
        }
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        for (i, p) in self.points.iter().enumerate() {
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

        changed |= ui
            .add(egui::Checkbox::new(
                &mut self.show_duplicates,
                "Show Duplicates",
            ))
            .changed();

        if changed {
            *self.duplicate_cache.write().unwrap() = None; // cache törlése
        }

        changed
    }

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        Some((self.center_x, self.center_y, self.radius))
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
        let (cx, cy) = (self.center_x, self.center_y);
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
            self.points.push(Point {
                x: cx + dx,
                y: cy + dy,
            });
            self.point_radii.push(r);
        }
    }
}
