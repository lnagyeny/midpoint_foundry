use crate::algorithms::ellipse::ellipse_base::EllipseBase;
use crate::algorithms::{Algorithm, Point};

pub struct MidpointEllipse {
    pub base: EllipseBase,
}

impl Default for MidpointEllipse {
    fn default() -> Self {
        let mut s = Self {
            base: EllipseBase::new(0, 0, 30, 20),
        };
        s.compute();
        s
    }
}

impl Algorithm for MidpointEllipse {
    fn name(&self) -> &str {
        "Midpoint Ellipse"
    }

    fn compute(&mut self) {
        self.base.points.clear();

        let mut x = 0;
        let mut y = self.base.radius_y;

        let mut d1 = (self.base.radius_y * self.base.radius_y
            - self.base.radius_x * self.base.radius_x * self.base.radius_y)
            as f64
            + 0.25 * (self.base.radius_x * self.base.radius_x) as f64;
        let mut dx = 2 * self.base.radius_y * self.base.radius_y * x;
        let mut dy = 2 * self.base.radius_x * self.base.radius_x * y;

        // Region 1
        while dx < dy {
            self.base.plot_symmetric(x, y, 0);
            if d1 < 0.0 {
                x += 1;
                dx += 2 * self.base.radius_y * self.base.radius_y;
                d1 += dx as f64 + (self.base.radius_y * self.base.radius_y) as f64;
            } else {
                x += 1;
                y -= 1;
                dx += 2 * self.base.radius_y * self.base.radius_y;
                dy -= 2 * self.base.radius_x * self.base.radius_x;
                d1 += (dx - dy + self.base.radius_y * self.base.radius_y) as f64;
            }
        }

        let mut d2 = (self.base.radius_y * self.base.radius_y) as f64
            * (x as f64 + 0.5)
            * (x as f64 + 0.5)
            + (self.base.radius_x * self.base.radius_x) as f64 * (y - 1) as f64 * (y - 1) as f64
            - (self.base.radius_x * self.base.radius_x) as f64
                * (self.base.radius_y * self.base.radius_y) as f64;

        // Region 2
        while y >= 0 {
            self.base.plot_symmetric(x, y, 0);
            if d2 > 0.0 {
                y -= 1;
                dy -= 2 * self.base.radius_x * self.base.radius_x;
                d2 += (self.base.radius_x * self.base.radius_x) as f64 - dy as f64;
            } else {
                y -= 1;
                x += 1;
                dx += 2 * self.base.radius_y * self.base.radius_y;
                dy -= 2 * self.base.radius_x * self.base.radius_x;
                d2 += (dx - dy + self.base.radius_x * self.base.radius_x) as f64;
            }
        }
    }

    fn points(&self) -> &[Point] {
        &self.base.points
    }

    fn color(&self) -> [f32; 3] {
        [0.2, 0.2, 0.38]
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.base.draw_ui_with_overlay(ui)
    }

    fn overlay_ellipse(&self) -> Option<(i32, i32, i32, i32)> {
        self.base.overlay_ellipse_base()
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        self.base.cell_info_base(cx, cy)
    }

    fn category(&self) -> &'static str {
        "ellipse"
    }
}
