use crate::algorithms::{Algorithm, Point};

pub struct MidpointLine {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    points: Vec<Point>,
}

impl Default for MidpointLine {
    fn default() -> Self {
        let mut s = Self {
            x0: -10,
            y0: -5,
            x1: 15,
            y1: 10,
            points: Vec::new(),
        };
        s.compute();
        s
    }
}

impl Algorithm for MidpointLine {
    fn name(&self) -> &str {
        "Midpoint Line"
    }

    fn compute(&mut self) {
        self.points.clear();

        let dx = (self.x1 - self.x0).abs();
        let dy = (self.y1 - self.y0).abs();
        let sx = if self.x0 < self.x1 { 1 } else { -1 };
        let sy = if self.y0 < self.y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let (mut x, mut y) = (self.x0, self.y0);

        loop {
            self.points.push(Point { x, y });
            if x == self.x1 && y == self.y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn points(&self) -> &[Point] {
        &self.points
    }

    fn color(&self) -> [f32; 3] {
        [1.0, 0.6, 0.0]
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.label("Start point");
        ui.horizontal(|ui| {
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.x0)
                        .prefix("X0: ")
                        .range(-100..=100),
                )
                .changed();
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.y0)
                        .prefix("Y0: ")
                        .range(-100..=100),
                )
                .changed();
        });

        ui.label("End point");
        ui.horizontal(|ui| {
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.x1)
                        .prefix("X1: ")
                        .range(-100..=100),
                )
                .changed();
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.y1)
                        .prefix("Y1: ")
                        .range(-100..=100),
                )
                .changed();
        });

        changed
    }
    fn category(&self) -> &'static str {
        "line"
    }
}
