use crate::algorithms::circle::circle_base::CircleBase;
use crate::algorithms::{Algorithm, Point};

pub struct ScanlineCircle {
    pub base: CircleBase,
}

impl Default for ScanlineCircle {
    fn default() -> Self {
        let mut s = Self {
            base: CircleBase::new(0, 0, 15),
        };
        s.compute();
        s
    }
}

impl Algorithm for ScanlineCircle {
    fn name(&self) -> &str {
        "Scanline Circle"
    }

    fn category(&self) -> &'static str {
        "circle"
    }

    fn compute(&mut self) {
        self.base.points.clear();

        let mut x = 0i32;
        let mut y = self.base.radius;
        let mut d = 1 - self.base.radius;

        while x <= y {
            self.fill_scanline(
                self.base.center_y + y,
                self.base.center_x - x,
                self.base.center_x + x,
            );
            self.fill_scanline(
                self.base.center_y - y,
                self.base.center_x - x,
                self.base.center_x + x,
            );

            self.fill_scanline(
                self.base.center_y + x,
                self.base.center_x - y,
                self.base.center_x + y,
            );
            self.fill_scanline(
                self.base.center_y - x,
                self.base.center_x - y,
                self.base.center_x + y,
            );

            if d < 0 {
                d += 2 * x + 3;
            } else {
                d += 2 * (x - y) + 5;
                y -= 1;
            }
            x += 1;
        }

        //self.points.sort_unstable_by_key(|p| (p.y, p.x));
        //self.points.dedup();
    }
    fn points(&self) -> &[Point] {
        &self.base.points
    }

    fn color(&self) -> [f32; 3] {
        [0.0, 0.78, 1.0]
    }

    fn point_color_override(&self, i: usize) -> Option<[f32; 3]> {
        self.base.duplicate_color(i, None)
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.base.draw_ui_with_duplicates(ui)
    }

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        self.base.overlay_circle_base()
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        self.base.cell_info_base(cx, cy)
    }
}

impl ScanlineCircle {
    fn fill_scanline(&mut self, row: i32, x_start: i32, x_end: i32) {
        for px in x_start..=x_end {
            self.base.points.push(Point { x: px, y: row });
        }
    }
}
