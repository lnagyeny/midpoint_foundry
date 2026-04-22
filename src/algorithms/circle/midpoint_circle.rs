use crate::algorithms::circle::circle_base::CircleBase;
use crate::algorithms::{Algorithm, Point};

pub struct MidpointCircle {
    pub base: CircleBase,
    points: Vec<Point>,
}

impl Default for MidpointCircle {
    fn default() -> Self {
        let mut s = Self {
            base: CircleBase::new(0, 0, 15),
            points: Vec::new(),
        };
        s.compute();
        s
    }
}

impl Algorithm for MidpointCircle {
    fn name(&self) -> &str {
        "Midpoint Circle"
    }

    fn category(&self) -> &'static str {
        "circle"
    }

    fn compute(&mut self) {
        self.points.clear();
        let mut x = 0i32;
        let mut y = self.base.radius;
        let mut d = 1 - self.base.radius;

        while x <= y {
            self.plot_octants(x, y);
            if d < 0 {
                d += 2 * x + 3;
            } else {
                d += 2 * (x - y) + 5;
                y -= 1;
            }
            x += 1;
        }
    }

    fn points(&self) -> &[Point] {
        &self.points
    }

    fn color(&self) -> [f32; 3] {
        [0.0, 0.78, 1.0]
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.base.draw_ui_base(ui)
    }

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        self.base.overlay_circle_base()
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        self.base.cell_info_base(cx, cy)
    }
}

impl MidpointCircle {
    fn plot_octants(&mut self, x: i32, y: i32) {
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
            self.points.push(Point {
                x: cx + dx,
                y: cy + dy,
            });
        }
    }
}
