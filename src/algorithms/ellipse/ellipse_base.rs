use crate::algorithms::Point;
use egui::Ui;
use std::collections::HashSet;
use std::sync::RwLock;

// ---------------------------------------------------------------------------
// Shared colour palettes
// ---------------------------------------------------------------------------

/// 7-colour accessible palette – used by algorithms that colour by ring index.
pub const PALETTE_7: [[f32; 3]; 7] = [
    [0.00, 0.45, 0.70], // deep blue
    [0.55, 0.35, 0.85], // purple
    [0.30, 0.70, 0.20], // green
    [0.80, 0.75, 0.00], // yellow-olive
    [0.00, 0.60, 0.50], // teal
    [0.90, 0.50, 0.00], // orange
    [0.40, 0.40, 0.40], // neutral gray
];

// ---------------------------------------------------------------------------
// EllipseBase
// ---------------------------------------------------------------------------

/// Shared state and helpers for every ellipse algorithm.
///
/// Each algorithm embeds this struct and delegates the common parts
/// (UI, overlay, cell info, duplicate highlighting) to it.
pub struct EllipseBase {
    pub center_x: i32,
    pub center_y: i32,
    pub radius_x: i32,
    pub radius_y: i32,

    /// Pixel buffer – algorithms write here directly.
    pub points: Vec<Point>,

    /// Concentric-ring index (1 … radius_y) for each point in `points`.
    /// Only filled by fill-style algorithms; outline algorithms can leave it empty.
    pub point_rings: Vec<i32>,

    /// When `true`, draw a smooth analytic ellipse on top of the pixel output.
    pub show_smooth_overlay: bool,

    /// When `true`, pixels that appear more than once are highlighted in red.
    pub show_duplicates: bool,
    pub duplicate_cache: RwLock<Option<HashSet<(i32, i32)>>>,
}

impl EllipseBase {
    // ------------------------------------------------------------------
    // Construction
    // ------------------------------------------------------------------

    pub fn new(center_x: i32, center_y: i32, radius_x: i32, radius_y: i32) -> Self {
        Self {
            center_x,
            center_y,
            radius_x,
            radius_y,
            points: Vec::new(),
            point_rings: Vec::new(),
            show_smooth_overlay: false,
            show_duplicates: false,
            duplicate_cache: RwLock::new(None),
        }
    }

    // ------------------------------------------------------------------
    // Helpers for algorithms
    // ------------------------------------------------------------------

    /// Push one point (and its ring index) in all four symmetric quadrants.
    ///
    /// Pass `ring = 0` and leave `point_rings` empty for outline algorithms
    /// that don't use ring colouring.
    pub fn plot_symmetric(&mut self, x: i32, y: i32, ring: i32) {
        for &(dx, dy) in &[(x, y), (-x, y), (x, -y), (-x, -y)] {
            self.points.push(Point {
                x: self.center_x + dx,
                y: self.center_y + dy,
            });
            self.point_rings.push(ring);
        }
    }

    /// Implicit ellipse equation value at pixel `(px, py)`:
    ///   b²·(px−cx)²  +  a²·(py−cy)²  −  a²·b²
    /// Negative ⟹ inside, zero ⟹ on curve, positive ⟹ outside.
    pub fn decision_parameter(&self, px: i32, py: i32) -> i64 {
        let dx = (px - self.center_x) as i64;
        let dy = (py - self.center_y) as i64;
        let a = self.radius_x as i64;
        let b = self.radius_y as i64;
        b * b * dx * dx + a * a * dy * dy - a * a * b * b
    }

    // ------------------------------------------------------------------
    // Colour helpers
    // ------------------------------------------------------------------

    /// Return the ring colour for point `i` using a palette of length `palette_len`.
    /// Returns `None` when `point_rings` is empty (outline algorithms).
    pub fn ring_color<const N: usize>(
        &self,
        i: usize,
        palette: &[[f32; 3]; N],
    ) -> Option<[f32; 3]> {
        let r = *self.point_rings.get(i)? as usize;
        Some(palette[r % N])
    }

    /// Duplicate-aware colour for point `i`.
    ///
    /// * If duplicate highlighting is **off** → returns `fallback`.
    /// * If the pixel is a duplicate → returns red `[1, 0, 0]`.
    /// * Otherwise → returns `fallback`.
    ///
    /// The internal cache is built lazily on the first call after a recompute.
    pub fn duplicate_color(&self, i: usize, fallback: Option<[f32; 3]>) -> Option<[f32; 3]> {
        if !self.show_duplicates {
            return fallback;
        }

        // Lazy-init duplicate set
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
            fallback
        }
    }

    /// Invalidate the duplicate cache (call whenever `points` is rebuilt).
    pub fn invalidate_cache(&self) {
        *self.duplicate_cache.write().unwrap() = None;
    }

    // ------------------------------------------------------------------
    // egui UI
    // ------------------------------------------------------------------

    /// Basic parameter controls: center X/Y and radius X/Y.
    pub fn draw_ui_base(&mut self, ui: &mut Ui) -> bool {
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

        ui.label("Radii");
        ui.horizontal(|ui| {
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.radius_x)
                        .prefix("X: ")
                        .range(1..=100),
                )
                .changed();
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.radius_y)
                        .prefix("Y: ")
                        .range(1..=100),
                )
                .changed();
        });

        changed
    }

    /// `draw_ui_base` + analytic overlay toggle (for outline algorithms).
    pub fn draw_ui_with_overlay(&mut self, ui: &mut Ui) -> bool {
        let changed = self.draw_ui_base(ui);
        ui.checkbox(
            &mut self.show_smooth_overlay,
            "Draw analytic ellipse overlay",
        );
        changed
    }

    /// `draw_ui_base` + duplicate-highlight toggle with cache invalidation.
    pub fn draw_ui_with_duplicates(&mut self, ui: &mut Ui) -> bool {
        let mut changed = self.draw_ui_base(ui);
        changed |= ui
            .add(egui::Checkbox::new(
                &mut self.show_duplicates,
                "Highlight duplicates",
            ))
            .changed();
        if changed {
            self.invalidate_cache();
        }
        changed
    }

    /// `draw_ui_base` + both extra toggles (overlay + duplicates).
    pub fn draw_ui_full(&mut self, ui: &mut Ui) -> bool {
        let mut changed = self.draw_ui_base(ui);
        ui.checkbox(
            &mut self.show_smooth_overlay,
            "Draw analytic ellipse overlay",
        );
        changed |= ui
            .add(egui::Checkbox::new(
                &mut self.show_duplicates,
                "Highlight duplicates",
            ))
            .changed();
        if changed {
            self.invalidate_cache();
        }
        changed
    }

    // ------------------------------------------------------------------
    // Algorithm trait helpers
    // ------------------------------------------------------------------

    /// Returns `Some((cx, cy, rx, ry))` when the overlay is enabled.
    /// Forward this from `Algorithm::overlay_ellipse`.
    pub fn overlay_ellipse_base(&self) -> Option<(i32, i32, i32, i32)> {
        if self.show_smooth_overlay {
            Some((self.center_x, self.center_y, self.radius_x, self.radius_y))
        } else {
            None
        }
    }

    /// Human-readable implicit-equation value for a hovered cell.
    /// Forward this from `Algorithm::cell_info`.
    pub fn cell_info_base(&self, cx: i32, cy: i32) -> Option<String> {
        let d = self.decision_parameter(cx, cy);
        Some(format!(
            "b²x²+a²y²−a²b² = {d}\n({}  ellipse)",
            if d < 0 {
                "inside"
            } else if d == 0 {
                "on"
            } else {
                "outside"
            }
        ))
    }

    pub fn cell_info_ring_base(&self, cx: i32, cy: i32) -> Option<String> {
        // Find which ring the clicked cell belongs to
        let ring = self
            .points
            .iter()
            .zip(self.point_rings.iter())
            .find(|(p, _)| p.x == cx && p.y == cy)
            .map(|(_, &r)| r)?;

        // n = ring - 1, mert ring 1 = outermost (i=0, tehát ry - 0 = ry)
        let n = ring - 1;
        let eff_rx = (self.radius_x - n) as f64;
        let eff_ry = (self.radius_y - n) as f64;

        // x koordináta a center-hez képest
        let x = (cx - self.center_x) as f64;

        let inner = 1.0 - (x * x) / (eff_rx * eff_rx);
        let y_val = if inner >= 0.0 {
            format!("{:.3}", eff_ry * inner.sqrt())
        } else {
            "– (x outside ellipse)".to_string()
        };

        Some(format!(
            "ring n = {n}\ny({n}, {x}) = ({eff_ry}) · √(1 − {x}²/{eff_rx}²)\n       = {y_val}"
        ))
    }
}
