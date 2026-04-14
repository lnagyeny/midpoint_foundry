use super::{Algorithm, Point};

pub struct RolandFillEllipse {
    pub center_x: i32,
    pub center_y: i32,
    pub radius_x: i32,
    pub radius_y: i32,
    points: Vec<Point>,
    /// Which concentric ring each point belongs to (1 … radius_x or radius_y).
    point_rings: Vec<i32>,
}

impl Default for RolandFillEllipse {
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

impl Algorithm for RolandFillEllipse {
    fn name(&self) -> &str {
        "Roland Fill Ellipse"
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
}

impl RolandFillEllipse {
    fn compute_ring(&mut self, rx: i32, ry: i32, ring: i32) {
        let (cx, cy) = (self.center_x, self.center_y);
        let mut x: i64 = 0;
        let mut y: i64 = ry as i64;

        let rx2: i64 = (rx as i64) * (rx as i64);
        let ry2: i64 = (ry as i64) * (ry as i64);
        let two_rx2: i64 = 2 * rx2;
        let two_ry2: i64 = 2 * ry2;

        let mut px: i64 = 0;
        let mut py: i64 = two_rx2 * y;

        self.plot_symmetric(cx, cy, x as i32, y as i32, ring);

        let mut p1: i64 = (ry2 as f64 - rx2 as f64 * ry as f64 + 0.25 * rx2 as f64).round() as i64;

        while px < py {
            x += 1;
            px += two_ry2;
            if p1 < 0 {
                p1 += ry2 + px;
            } else {
                y -= 1;
                py -= two_rx2;
                p1 += ry2 + px - py;
            }
            self.plot_symmetric(cx, cy, x as i32, y as i32, ring);
        }

        let mut p2: f64 = ry2 as f64 * (x as f64 + 0.5) * (x as f64 + 0.5)
            + rx2 as f64 * (y - 1) as f64 * (y - 1) as f64
            - rx2 as f64 * ry2 as f64;

        while y > 0 {
            y -= 1;
            py -= two_rx2;
            if p2 > 0.0 {
                p2 += (rx2 - py) as f64;
            } else {
                x += 1;
                px += two_ry2;
                p2 += (rx2 - py + px) as f64;
            }
            self.plot_symmetric(cx, cy, x as i32, y as i32, ring);
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
