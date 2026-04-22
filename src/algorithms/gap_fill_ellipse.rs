use super::{Algorithm, Point};
use std::collections::HashSet;
use std::sync::RwLock;

pub struct GapFillEllipse {
    pub center_x: i32,
    pub center_y: i32,
    pub radius_x: i32,
    pub radius_y: i32,
    points: Vec<Point>,
    /// Which concentric ring each point belongs to (1 … radius_x or radius_y).
    point_rings: Vec<i32>,
    pub show_duplicates: bool,
    duplicate_cache: RwLock<Option<HashSet<(i32, i32)>>>,
}

impl Default for GapFillEllipse {
    fn default() -> Self {
        let mut s = Self {
            center_x: 0,
            center_y: 0,
            radius_x: 30,
            radius_y: 20,
            points: Vec::new(),
            point_rings: Vec::new(),
            show_duplicates: false,
            duplicate_cache: RwLock::new(None),
        };
        s.compute();
        s
    }
}

// 12-colour vibrant palette, indexed by `ring % 12`.
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

const PALETTE: [[f32; 3]; 7] = [
    [0.00, 0.45, 0.70], // deep blue
    [0.55, 0.35, 0.85], // purple
    [0.30, 0.70, 0.20], // green
    [0.80, 0.75, 0.00], // yellow-olive
    [0.00, 0.60, 0.50], // teal
    [0.90, 0.50, 0.00], // orange (not too redish)
    [0.40, 0.40, 0.40], // neutral gray (good for contrast reference);
];

impl Algorithm for GapFillEllipse {
    fn name(&self) -> &str {
        "Gap-Fill Ellipse"
    }

    fn compute(&mut self) {
        self.points.clear();
        self.point_rings.clear();

        let max_ry = self.radius_y;
        for i in 0..=max_ry {
            let rx = self.radius_x - i;
            let ry = self.radius_y - i;
            if rx >= 1 && ry >= 1 {
                self.compute_ring(rx, ry, i + 1);
            }
        }
    }

    fn points(&self) -> &[Point] {
        &self.points
    }

    fn color(&self) -> [f32; 3] {
        [0.2, 0.2, 0.38]
    }

    fn point_color_override(&self, i: usize) -> Option<[f32; 3]> {
        if !self.show_duplicates {
            let r = *self.point_rings.get(i)? as usize;
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
            let r = *self.point_rings.get(i)? as usize;
            Some(PALETTE[r % 3])
        }
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
}

impl GapFillEllipse {
    fn compute_ring(&mut self, rx: i32, ry: i32, ring: i32) {
        let (cx, cy) = (self.center_x, self.center_y);

        let mut x = 0i32;
        let mut y = ry;

        let mut d1 = (ry * ry - rx * rx * ry) as f64 + 0.25 * (rx * rx) as f64;
        let mut dx = 2 * ry * ry * x;
        let mut dy = 2 * rx * rx * y;

        // Region 1
        while dx < dy {
            self.plot_symmetric(cx, cy, x, y, ring);
            if d1 < 0.0 {
                x += 1;
                dx += 2 * ry * ry;
                d1 += dx as f64 + (ry * ry) as f64;
            } else {
                x += 1;
                y -= 1;
                dx += 2 * ry * ry;
                dy -= 2 * rx * rx;
                d1 += (dx - dy + ry * ry) as f64;
            }
        }

        let mut d2 = (ry * ry) as f64 * (x as f64 + 0.5) * (x as f64 + 0.5)
            + (rx * rx) as f64 * (y - 1) as f64 * (y - 1) as f64
            - (rx * rx) as f64 * (ry * ry) as f64;

        // Region 2
        while y >= 0 {
            self.plot_symmetric(cx, cy, x, y, ring);
            if d2 > 0.0 {
                y -= 1;
                dy -= 2 * rx * rx;
                d2 += (rx * rx) as f64 - dy as f64;
            } else {
                y -= 1;
                x += 1;
                dx += 2 * ry * ry;
                dy -= 2 * rx * rx;
                d2 += (dx - dy + rx * rx) as f64;
            }
        }
    }

    fn plot_symmetric(&mut self, cx: i32, cy: i32, x: i32, y: i32, ring: i32) {
        for &(dx, dy) in &[(x, y), (-x, y), (x, -y), (-x, -y)] {
            self.points.push(Point {
                x: cx + dx,
                y: cy + dy,
            });
            self.point_rings.push(ring);
        }
    }
}
