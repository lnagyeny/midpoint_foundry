use crate::algorithms::ellipse::ellipse_base;
use crate::algorithms::ellipse::ellipse_base::EllipseBase;
use crate::algorithms::{Algorithm, Point};

pub struct GapFillEllipse {
    pub base: EllipseBase,
}

impl Default for GapFillEllipse {
    fn default() -> Self {
        let mut s = Self {
            base: EllipseBase::new(0, 0, 30, 20),
        };
        s.compute();
        s
    }
}

impl Algorithm for GapFillEllipse {
    fn name(&self) -> &str {
        "Gap-Fill Ellipse"
    }

    fn compute(&mut self) {
        self.base.points.clear();
        self.base.point_rings.clear();

        let max_ry = self.base.radius_y;
        for i in 0..=max_ry {
            let rx = self.base.radius_x - i;
            let ry = self.base.radius_y - i;
            if rx >= 1 && ry >= 1 {
                self.compute_ring(rx, ry, i + 1);
            }
        }
    }

    fn points(&self) -> &[Point] {
        &self.base.points
    }

    fn color(&self) -> [f32; 3] {
        [0.2, 0.2, 0.38]
    }

    fn point_color_override(&self, i: usize) -> Option<[f32; 3]> {
        let fallback = {
            let r = *self.base.point_rings.get(i)? as usize;
            Some(ellipse_base::PALETTE_7[r % 7])
        };
        self.base.duplicate_color(i, fallback)
    }

    fn overlay_ellipse(&self) -> Option<(i32, i32, i32, i32)> {
        self.base.overlay_ellipse_base()
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        self.base.cell_info_ring_base(cx, cy)
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.base.draw_ui_full(ui)
    }

    fn category(&self) -> &'static str {
        "ellipse"
    }
}

impl GapFillEllipse {
    fn compute_ring(&mut self, rx: i32, ry: i32, ring: i32) {
        let mut x = 0i32;
        let mut y = ry;

        let mut d1 = (ry * ry - rx * rx * ry) as f64 + 0.25 * (rx * rx) as f64;
        let mut dx = 2 * ry * ry * x;
        let mut dy = 2 * rx * rx * y;

        // Region 1
        while dx < dy {
            self.base.plot_symmetric(x, y, ring);
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
            self.base.plot_symmetric(x, y, ring);
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
}
