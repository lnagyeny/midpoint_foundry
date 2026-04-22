use crate::algorithms::circle::circle_base::CircleBase;
use crate::algorithms::{Algorithm, Point};
use std::collections::HashSet;
use std::sync::RwLock;

pub struct ScanlineCircle {
    pub base: CircleBase,
    points: Vec<Point>,
    pub show_duplicates: bool,
    duplicate_cache: RwLock<Option<HashSet<(i32, i32)>>>,
}

impl Default for ScanlineCircle {
    fn default() -> Self {
        let mut s = Self {
            base: CircleBase::new(0, 0, 15),
            points: Vec::new(),
            show_duplicates: false,
            duplicate_cache: RwLock::new(None),
        };
        s.compute();
        s
    }
}

impl Algorithm for ScanlineCircle {
    fn name(&self) -> &str {
        "Scanline Circle"
    }

    fn compute(&mut self) {
        self.points.clear();

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
        &self.points
    }

    fn color(&self) -> [f32; 3] {
        [0.0, 0.78, 1.0]
    }

    fn point_color_override(&self, i: usize) -> Option<[f32; 3]> {
        if !self.show_duplicates {
            return None;
        }
        // Lazy init
        if self.duplicate_cache.read().unwrap().is_none() {
            let mut seen = HashSet::new();
            let mut dupes = HashSet::new();
            for p in &self.points {
                if !seen.insert((p.x, p.y)) {
                    dupes.insert((p.x, p.y));
                }
            }
            *self.duplicate_cache.write().unwrap() = Some(dupes);
        }

        let cache = self.duplicate_cache.read().unwrap();
        let dupes = cache.as_ref().unwrap();
        let p = &self.points[i];
        if dupes.contains(&(p.x, p.y)) {
            Some([1.0, 0.0, 0.0])
        } else {
            None
        }
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = self.base.draw_ui_base(ui);

        changed |= ui
            .checkbox(&mut self.show_duplicates, "Highlight duplicates")
            .changed();

        if changed {
            *self.duplicate_cache.write().unwrap() = None; // cache törlése
        }
        changed
    }

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        self.base.overlay_circle_base()
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        self.base.cell_info_base(cx, cy)
    }

    fn category(&self) -> &'static str {
        "circle"
    }
}

impl ScanlineCircle {
    fn fill_scanline(&mut self, row: i32, x_start: i32, x_end: i32) {
        for px in x_start..=x_end {
            self.points.push(Point { x: px, y: row });
        }
    }
}
