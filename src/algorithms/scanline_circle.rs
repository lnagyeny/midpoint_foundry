use super::{Algorithm, Point};
use std::collections::HashSet;
use std::sync::RwLock;

pub struct ScanlineCircle {
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    pub show_smooth_overlay: bool,
    points: Vec<Point>,
    pub show_duplicates: bool,
    duplicate_cache: RwLock<Option<HashSet<(i32, i32)>>>,
}

impl Default for ScanlineCircle {
    fn default() -> Self {
        let mut s = Self {
            center_x: 0,
            center_y: 0,
            radius: 15,
            show_smooth_overlay: false,
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
        let mut y = self.radius;
        let mut d = 1 - self.radius;

        while x <= y {
            self.fill_scanline(self.center_y + y, self.center_x - x, self.center_x + x);
            self.fill_scanline(self.center_y - y, self.center_x - x, self.center_x + x);

            self.fill_scanline(self.center_y + x, self.center_x - y, self.center_x + y);
            self.fill_scanline(self.center_y - x, self.center_x - y, self.center_x + y);

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

        ui.checkbox(
            &mut self.show_smooth_overlay,
            "Draw analytic circle overlay",
        );

        changed |= ui
            .checkbox(&mut self.show_duplicates, "Highlight duplicates")
            .changed();

        if changed {
            *self.duplicate_cache.write().unwrap() = None; // cache törlése
        }
        changed
    }

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        if self.show_smooth_overlay {
            Some((self.center_x, self.center_y, self.radius))
        } else {
            None
        }
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        let d = self.decision_parameter(cx, cy);
        Some(format!(
            "x²+y²−r²  =  {d}\n({}  circle)",
            if d < 0 {
                "inside"
            } else if d == 0 {
                "on"
            } else {
                "outside"
            }
        ))
    }
}

impl ScanlineCircle {
    fn fill_scanline(&mut self, row: i32, x_start: i32, x_end: i32) {
        for px in x_start..=x_end {
            self.points.push(Point { x: px, y: row });
        }
    }

    /// Implicit circle equation: negative ⟹ inside, zero ⟹ on, positive ⟹ outside.
    pub fn decision_parameter(&self, x: i32, y: i32) -> i32 {
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        dx * dx + dy * dy - self.radius * self.radius
    }
}
