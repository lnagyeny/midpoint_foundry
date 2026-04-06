pub mod baseline_circle;
pub mod gap_fill_circle;
pub mod midpoint_circle;
pub mod midpoint_line;
pub mod parallel_midpoint_circle;
pub mod sitaraman_fill_circle;

// ── Shared data types ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default)]
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

    /// Optional: extra text shown in the panel when a cell is selected.
    fn cell_info(&self, _cx: i32, _cy: i32) -> Option<String> {
        None
    }
}
