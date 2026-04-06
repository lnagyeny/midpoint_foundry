use super::{Algorithm, Point};

pub struct FilledMidpointCircleReal {
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    points: Vec<Point>,
    /// Which concentric ring each point belongs to (1 … radius).
    point_radii: Vec<i32>,
}

impl Default for FilledMidpointCircleReal {
    fn default() -> Self {
        let mut s = Self {
            center_x: 0,
            center_y: 0,
            radius: 15,
            points: Vec::new(),
            point_radii: Vec::new(),
        };
        s.compute();
        s
    }
}

// 12-colour vibrant palette, indexed by `ring_radius % 12`.
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

impl Algorithm for FilledMidpointCircleReal {
    fn name(&self) -> &str {
        "Filled Midpoint Circle Real"
    }

    fn compute(&mut self) {
        self.points.clear();
        self.point_radii.clear();
        for r in 1..=self.radius {
            self.compute_ring(r);
        }
    }

    fn points(&self) -> &[Point] {
        &self.points
    }

    fn color(&self) -> [f32; 3] {
        [0.0, 0.5, 1.0]
    }

    fn point_color_override(&self, index: usize) -> Option<[f32; 3]> {
        let r = *self.point_radii.get(index)? as usize;
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

        changed |= ui
            .add(egui::Slider::new(&mut self.radius, 1..=100).text("Radius"))
            .changed();

        changed
    }
}

impl FilledMidpointCircleReal {
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
