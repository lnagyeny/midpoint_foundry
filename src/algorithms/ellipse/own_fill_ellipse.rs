use crate::algorithms::{Algorithm, Point};

pub struct OwnFillEllipse {
    pub center_x: i32,
    pub center_y: i32,
    pub radius_x: i32,
    pub radius_y: i32,
    points: Vec<Point>,
    /// Which concentric ring each point belongs to (1 … radius_x or radius_y).
    point_rings: Vec<i32>,
}

impl Default for OwnFillEllipse {
    fn default() -> Self {
        let mut s = Self {
            center_x: 0,
            center_y: 0,
            radius_x: 30,
            radius_y: 20,
            points: Vec::new(),
            point_rings: Vec::new(),
        };
        s.compute();
        s
    }
}

// 12-colour vibrant palette, indexed by `ring % 12`.
const PALETTE: [[f32; 3]; 12] = [
    [1.00, 0.00, 0.00], // red
    [0.00, 1.00, 0.00], // green
    [0.00, 0.00, 1.00], // blue
    [1.00, 1.00, 0.00], // yellow
    [1.00, 0.00, 1.00], // magenta
    [0.00, 1.00, 1.00], // cyan
    [1.00, 0.50, 0.00], // orange
    [0.50, 0.00, 1.00], // purple
    [0.00, 1.00, 0.50], // spring green
    [1.00, 0.00, 0.50], // rose
    [0.50, 1.00, 0.00], // lime
    [0.00, 0.50, 1.00], // sky blue
];

impl Algorithm for OwnFillEllipse {
    fn name(&self) -> &str {
        "Own Fill Ellipse"
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

    fn point_color_override(&self, index: usize) -> Option<[f32; 3]> {
        let r = *self.point_rings.get(index)? as usize;
        Some(PALETTE[r % 12])
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

        changed
    }

    fn category(&self) -> &'static str {
        "ellipse"
    }
}

impl OwnFillEllipse {
    fn compute_ring(&mut self, rx: i32, ry: i32, ring: i32) {
        let (cx, cy) = (self.center_x, self.center_y);

        let mut x = 0i32;
        let mut y = ry;

        let rx_sq = (rx * rx) as f64;
        let ry_sq = (ry * ry) as f64;

        // Kezdő döntési paraméter Region 1-ben
        let mut d1 = ry_sq - (rx_sq * ry as f64) + (0.25 * rx_sq);
        let mut dx = 2.0 * ry_sq * x as f64;
        let mut dy = 2.0 * rx_sq * y as f64;

        // Region 1 (Ahol a vízszintes haladás dominál)
        while dx < dy {
            self.plot_symmetric(cx, cy, x, y, ring);

            if d1 < 0.0 {
                x += 1;
                dx += 2.0 * ry_sq;
                d1 += dx + ry_sq;
            } else {
                // JAVÍTOTT SITARAMAN-FELTÉTEL (Region 1)
                // Itt d1 = f(x+1, y-0.5). Azt nézzük, hogy f(x+1, y) belefér-e még.
                // f(x+1, y) = d1 + rx_sq * y - 0.25 * rx_sq
                if (d1 + rx_sq * y as f64 - 0.25 * rx_sq) < (rx_sq * 0.5) {
                    self.plot_symmetric(cx, cy, x + 1, y, ring);
                }

                x += 1;
                y -= 1;
                dx += 2.0 * ry_sq;
                dy -= 2.0 * rx_sq;
                d1 += dx - dy + ry_sq;
            }
        }

        // Kezdő döntési paraméter Region 2-ben
        let mut d2 = ry_sq * ((x as f64 + 0.5) * (x as f64 + 0.5))
            + rx_sq * ((y - 1) as f64 * (y - 1) as f64)
            - rx_sq * ry_sq;

        // Region 2 (Ahol a függőleges haladás dominál)
        while y >= 0 {
            self.plot_symmetric(cx, cy, x, y, ring);

            if d2 > 0.0 {
                y -= 1;
                dy -= 2.0 * rx_sq;
                d2 += rx_sq - dy;
            } else {
                // JAVÍTOTT SITARAMAN-FELTÉTEL (Region 2)
                // Itt d2 = f(x+0.5, y-1). Azt nézzük, hogy f(x, y-1) belefér-e még.
                // f(x, y-1) = d2 + ry_sq * x - 0.25 * ry_sq
                if (d2 + ry_sq * x as f64 - 0.25 * ry_sq) < (ry_sq * 0.5) {
                    self.plot_symmetric(cx, cy, x, y - 1, ring);
                }

                y -= 1;
                x += 1;
                dx += 2.0 * ry_sq;
                dy -= 2.0 * rx_sq;
                d2 += dx - dy + rx_sq;
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
