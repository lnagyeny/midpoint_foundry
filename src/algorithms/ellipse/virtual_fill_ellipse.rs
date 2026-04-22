use crate::algorithms::{Algorithm, Point};

pub struct VirtualFillEllipse {
    pub center_x: i32,
    pub center_y: i32,
    pub radius_x: i32,
    pub radius_y: i32,
    points: Vec<Point>,
    /// Which concentric ring each point belongs to (1 … radius_x or radius_y).
    point_rings: Vec<i32>,
}

impl Default for VirtualFillEllipse {
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

impl Algorithm for VirtualFillEllipse {
    fn name(&self) -> &str {
        "Virtual Fill Ellipse"
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

impl VirtualFillEllipse {
    fn compute_ring(&mut self, rx: i32, ry: i32, ring: i32) {
        let (cx, cy) = (self.center_x, self.center_y);

        let mut x = 0i32;
        let mut y = ry;

        // outer radii for gap detection
        let orx = rx + 1;
        let ory = ry + 1;

        // inner parameters
        let mut d1 = (ry * ry) as f64 - (rx * rx * ry) as f64 + 0.25 * (rx * rx) as f64;
        let mut dx = 2 * ry * ry * x;
        let mut dy = 2 * rx * rx * y;

        // outer parameters
        let mut od1 = (ory * ory) as f64 - (orx * orx * ory) as f64 + 0.25 * (orx * orx) as f64;
        let mut odx = 2 * ory * ory * x;
        let mut ody = 2 * orx * orx * y;

        // Region 1
        while dx < dy {
            self.plot_symmetric(cx, cy, x, y, ring);

            if d1 < 0.0 {
                // both would step horizontally
                x += 1;
                dx += 2 * ry * ry;
                d1 += dx as f64 + (ry * ry) as f64;

                // update outer (follows inner)
                odx += 2 * ory * ory;
                od1 += odx as f64 + (ory * ory) as f64;
            } else {
                // inner would step diagonally (y-1, x+1)
                // check the outer before stepping
                if od1 < 0.0 {
                    // the outer would still step horizontally -> gap
                    self.plot_symmetric(cx, cy, x + 1, y, ring);
                }

                x += 1;
                y -= 1;
                dx += 2 * ry * ry;
                dy -= 2 * rx * rx;
                d1 += (dx - dy + ry * ry) as f64;

                // update outer (follows inner)
                odx += 2 * ory * ory;
                ody -= 2 * orx * orx;
                od1 += (odx - ody + ory * ory) as f64;
            }
        }

        // Region 2 init
        let mut d2 = (ry * ry) as f64 * (x as f64 + 0.5).powi(2)
            + (rx * rx) as f64 * (y - 1) as f64 * (y - 1) as f64
            - (rx * rx * ry * ry) as f64;

        let mut od2 = (ory * ory) as f64 * (x as f64 + 0.5).powi(2)
            + (orx * orx) as f64 * (y - 1) as f64 * (y - 1) as f64
            - (orx * orx * ory * ory) as f64;

        // Region 2
        while y >= 0 {
            self.plot_symmetric(cx, cy, x, y, ring);

            if d2 > 0.0 {
                // both would step vertically
                y -= 1;
                dy -= 2 * rx * rx;
                d2 += (rx * rx) as f64 - dy as f64;

                ody -= 2 * orx * orx;
                od2 += (orx * orx) as f64 - ody as f64;
            } else {
                // inner would step diagonally (y-1, x+1)
                // check the outer before stepping
                if od2 > 0.0 {
                    // the outer would still step vertically -> gap
                    self.plot_symmetric(cx, cy, x, y - 1, ring);
                }

                y -= 1;
                x += 1;
                dx += 2 * ry * ry;
                dy -= 2 * rx * rx;
                d2 += (dx - dy + rx * rx) as f64;

                odx += 2 * ory * ory;
                ody -= 2 * orx * orx;
                od2 += (odx - ody + orx * orx) as f64;
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
