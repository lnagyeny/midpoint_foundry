pub mod circle;
pub mod ellipse;
pub mod line;
pub mod parabola;

// ── Shared data types ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

// ── Algorithm trait ───────────────────────────────────────────────────────────

pub trait Algorithm: Send + Sync {
    fn name(&self) -> &str;
    fn compute(&mut self);
    fn points(&self) -> &[Point];

    /// Uniform colour for all points.  Ignored when `point_color_override`
    /// returns `Some`.
    fn color(&self) -> [f32; 3] {
        [1.0, 0.0, 0.0]
    }

    /// Per-point colour override; return `None` to fall back to `color()`.
    fn point_color_override(&self, _index: usize) -> Option<[f32; 3]> {
        None
    }

    /// Draw algorithm-specific parameter widgets inside the right-hand panel.
    /// Return `true` when a parameter change requires an immediate recompute.
    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool;

    /// Optional analytic circle overlay: `Some((cx, cy, radius))`.
    fn overlay_circle(&self) -> Option<(i32, i32, i32)> {
        None
    }

    /// Optional analytic parabola overlay: `Some((vertex_x, vertex_y, p))`.
    fn overlay_parabola(&self) -> Option<(i32, i32, i32)> {
        None
    }

    /// Optional analytic ellipse overlay: `Some((cx, cy, radius_x, radius_y))`.
    fn overlay_ellipse(&self) -> Option<(i32, i32, i32, i32)> {
        None
    }

    /// Optional: extra text shown in the panel when a cell is selected.
    fn cell_info(&self, _cx: i32, _cy: i32) -> Option<String> {
        None
    }

    /// Get the category/shape of this algorithm (e.g., "circle", "ellipse", "line", "parabola").
    fn category(&self) -> &'static str;
}
