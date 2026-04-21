use super::{Algorithm, Point};

pub struct SitaramanFillCircle {
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    points: Vec<Point>,
    /// Which concentric ring each point belongs to (1 … radius).
    point_radii: Vec<i32>,
    /// Starting index of each ring in the points array (0-based).
    ring_start_indices: Vec<usize>,
}

impl Default for SitaramanFillCircle {
    fn default() -> Self {
        let mut s = Self {
            center_x: 0,
            center_y: 0,
            radius: 15,
            points: Vec::new(),
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

    fn point_color_override(&self, index: usize) -> Option<[f32; 3]> {
        let r = *self.point_radii.get(index)? as usize;
        Some(PALETTE[r % 3])
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

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        Some((self.center_x, self.center_y, self.radius))
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
