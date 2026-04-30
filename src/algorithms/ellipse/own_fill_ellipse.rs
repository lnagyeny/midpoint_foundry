use crate::algorithms::ellipse::ellipse_base;
use crate::algorithms::ellipse::ellipse_base::EllipseBase;
use crate::algorithms::{Algorithm, Point};

pub struct OwnFillEllipse {
    pub base: EllipseBase,
}

impl Default for OwnFillEllipse {
    fn default() -> Self {
        let mut s = Self {
            base: EllipseBase::new(0, 0, 30, 20),
        };
        s.compute();
        s
    }
}

impl Algorithm for OwnFillEllipse {
    fn name(&self) -> &str {
        "Own Fill Ellipse"
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
        // middle horizontal line to fill the center
        let final_rx = self.base.radius_x - (self.base.radius_y - 1);
        for x in -final_rx..=final_rx {
            self.base.points.push(Point { x, y: 0 });
            self.base.point_rings.push(max_ry + 1);
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

impl OwnFillEllipse {
    fn compute_ring(&mut self, rx: i32, ry: i32, ring: i32) {
        let mut x: i64 = 0;
        let mut y: i64 = ry as i64;
        let rx2: i64 = (rx as i64) * (rx as i64);
        let ry2: i64 = (ry as i64) * (ry as i64);
        let two_rx2: i64 = 2 * rx2;
        let two_ry2: i64 = 2 * ry2;
        let mut px: i64 = 0;
        let mut py: i64 = two_rx2 * y;
        self.base.plot_symmetric(x as i32, y as i32, ring);
        let mut p1: i64 = (ry2 as f64 - rx2 as f64 * ry as f64 + 0.25 * rx2 as f64).round() as i64;
        while px < py {
            x += 1;
            px += two_ry2;

            if p1 < 0 {
                p1 += ry2 + px;
            } else {
                let old_y = y;

                y -= 1;
                py -= two_rx2;
                p1 += ry2 + px - py;

                // bridge pixel
                self.base.plot_symmetric(x as i32, old_y as i32, ring);
            }

            self.base.plot_symmetric(x as i32, y as i32, ring);
        }
        let mut p2: f64 = ry2 as f64 * (x as f64 + 0.5) * (x as f64 + 0.5)
            + rx2 as f64 * (y - 1) as f64 * (y - 1) as f64
            - rx2 as f64 * ry2 as f64;
        while y > 0 {
            let old_x = x;

            y -= 1;
            py -= two_rx2;

            if p2 > 0.0 {
                p2 += (rx2 - py) as f64;
            } else {
                x += 1;
                px += two_ry2;
                p2 += (rx2 - py + px) as f64;

                // bridge pixel
                self.base.plot_symmetric(old_x as i32, y as i32, ring);
            }

            self.base.plot_symmetric(x as i32, y as i32, ring);
        }
    }
}
