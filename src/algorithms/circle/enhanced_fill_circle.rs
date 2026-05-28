use super::super::{Algorithm, Point};
use super::circle_base::{CircleBase, PALETTE};

/// Circle fill using the enhanced MPCDA from Nithya & Idrisi (2024).
///
/// The paper introduces a variable grid-width 'h' into the midpoint circle
/// decision parameter.  Smaller h values trace the analytic circle more
/// closely before rounding to integer pixels, reducing rasterisation error.
///
/// Key recurrences (Section 2 of the paper):
///   Initial:  Pk = h(h − r)
///   Pk < 0  → next pixel (x+h, y),   Pk+1 = Pk + 2h·x + 3h²
///   Pk ≥ 0  → next pixel (x+h, y−h), Pk+1 = Pk + 2h(x − y + h) + 3h²
///   x always increments by h; loop while x ≤ y  (first octant).
///
/// The filled disc is built by running the algorithm independently for every
/// concentric ring r = 1 … radius, mirroring the original behaviour.
pub struct EnhancedFillCircle {
    pub base: CircleBase,
    /// Grid-width h from the paper (h = 1.0 ↔ classical MPCDA).
    pub h: f64,
    /// Which concentric ring each point belongs to (1 … radius).
    point_radii: Vec<i32>,
    /// Starting index of each ring in the points array (0-based).
    ring_start_indices: Vec<usize>,
}

impl Default for EnhancedFillCircle {
    fn default() -> Self {
        let mut s = Self {
            base: CircleBase::new(0, 0, 15),
            h: 1.0,
            point_radii: Vec::new(),
            ring_start_indices: Vec::new(),
        };
        s.compute();
        s
    }
}

impl Algorithm for EnhancedFillCircle {
    fn name(&self) -> &str {
        "Enhanced MPCDA (Nithya & Idrisi 2024)"
    }

    fn compute(&mut self) {
        self.base.points.clear();
        self.point_radii.clear();
        self.ring_start_indices.clear();

        for r in 1..=self.base.radius {
            self.ring_start_indices.push(self.base.points.len());
            self.compute_ring(r);
        }
    }

    fn points(&self) -> &[Point] {
        &self.base.points
    }

    fn color(&self) -> [f32; 3] {
        [0.0, 0.5, 1.0]
    }

    fn point_color_override(&self, i: usize) -> Option<[f32; 3]> {
        let ring_color = {
            let r = *self.point_radii.get(i)? as usize;
            Some(PALETTE[r % PALETTE.len()])
        };
        self.base.duplicate_color(i, ring_color)
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = self.base.draw_ui_with_duplicates(ui);

        ui.separator();

        ui.label("Pixel-width h (Nithya & Idrisi, 2024):");

        let old_h = self.h;

        // Fine-grained slider
        ui.add(
            egui::Slider::new(&mut self.h, 0.05_f64..=1.0_f64)
                .step_by(0.05)
                .text("h"),
        );

        // Preset buttons matching the paper's three studied values
        ui.horizontal(|ui| {
            for &preset in &[1.0_f64, 0.5, 0.1] {
                let selected = (self.h - preset).abs() < 1e-9;
                if ui
                    .selectable_label(selected, format!("h = {preset}"))
                    .clicked()
                {
                    self.h = preset;
                }
            }
        });

        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(
                "h = 1.0 → classic MPCDA\n\
                 h < 1.0 → smoother steps, fewer errors\n",
            )
            .weak()
            .small(),
        );

        // Detect h change; invalidate duplicate cache accordingly.
        if (self.h - old_h).abs() > 1e-12 {
            changed = true;
            *self.base.duplicate_cache.write().unwrap() = None;
        }

        changed
    }

    fn category(&self) -> &'static str {
        "circle"
    }

    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        self.base.overlay_circle_base()
    }
}

impl EnhancedFillCircle {
    /// Run the Nithya & Idrisi algorithm for a single ring of radius `ring_r`.
    ///
    /// The algorithm works in floating-point using step h, then rounds each
    /// computed point to the nearest integer pixel before storing it.
    fn compute_ring(&mut self, ring_r: i32) {
        let r = ring_r as f64;
        // Guard: h must be positive and at most r (otherwise the loop body
        // would be skipped entirely, or y would immediately go negative).
        let h = self.h.clamp(0.01, r.max(0.01));

        let mut x = 0.0_f64;
        let mut y = r;
        // Initial decision parameter – Eq. (5) in the paper:
        //   Pk = h² + (r − h/2)² − r² ≈ h(h − r)
        let mut pk = h * (h - r);

        while x <= y + 1e-9 {
            self.plot_octants(x, y, ring_r);

            if pk >= 0.0 {
                // Choose pixel (x+h, y−h).
                // Pk+1 = Pk + 2h(x − y + h) + 3h²   [derived from Eq. 4 & 6]
                pk += 2.0 * h * (x - y + h) + 3.0 * h * h;
                y -= h;
            } else {
                // Choose pixel (x+h, y).
                // Pk+1 = Pk + 2h·x + 3h²             [derived from Eq. 4 & 7]
                pk += 2.0 * h * x + 3.0 * h * h;
            }
            x += h;
        }
    }

    /// Plot all eight symmetric octants for a floating-point (x, y) pair.
    ///
    /// Non-integer coordinates are rounded to the nearest pixel; this is where
    /// the paper's accuracy gain manifests: with smaller h the chosen real-valued
    /// points lie closer to the true circle before rounding.
    fn plot_octants(&mut self, x: f64, y: f64, r: i32) {
        let (cx, cy) = (self.base.center_x, self.base.center_y);
        let xi = x.round() as i32;
        let yi = y.round() as i32;

        for &(dx, dy) in &[
            (xi, yi),
            (-xi, yi),
            (xi, -yi),
            (-xi, -yi),
            (yi, xi),
            (-yi, xi),
            (yi, -xi),
            (-yi, -xi),
        ] {
            self.base.points.push(Point {
                x: cx + dx,
                y: cy + dy,
            });
            self.point_radii.push(r);
        }
    }
}
