pub mod circle;
pub mod ellipse;
pub mod hex;
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

    /// Optional complete custom geometry for **research experiments**.
    ///
    /// When this returns `Some`, `geometry::build` uses the supplied vertices
    /// verbatim and skips all square-grid drawing (grid lines, unit-square
    /// cells, overlays). This lets an isolated experiment (for example the
    /// hexagonal grid in `algorithms::hex`) emit arbitrary polygons such as
    /// hexagons without changing the square-grid rendering path.
    ///
    /// `mouse_cell` and `selected_cell` are passed through so an experiment can
    /// draw its own hover/selection highlight in its own coordinate system.
    ///
    /// Every existing square-grid algorithm inherits the default `None`, so
    /// their rendering is byte-for-byte unchanged.
    fn overlay_geometry(
        &self,
        _mouse_cell: Option<(i32, i32)>,
        _selected_cell: Option<(i32, i32)>,
    ) -> Option<crate::algorithms::hex::ExperimentGeometry> {
        None
    }

    /// Optional **viewport** override for a research experiment.
    ///
    /// Returns `(centre_x, centre_y, half_extent)` in *grid-space* units. When
    /// `Some`, the renderer maps the world-space square
    /// `[c−half, c+half]²` onto a **square screen viewport** (centred in the
    /// grid area) using a single uniform scale for both axes. This keeps
    /// cells regular and circles circular regardless of the window's
    /// width/height, at the cost of letterboxing when the grid area is not
    /// square.
    ///
    /// Default `None` keeps the original square-grid projection, which maps
    /// `[−grid_size/2, +grid_size/2]²` onto the full rectangular grid area.
    /// The default therefore leaves every existing algorithm unchanged.
    fn viewport(&self) -> Option<(f32, f32, f32)> {
        None
    }

    /// Optional: extra text shown in the panel when a cell is selected.
    fn cell_info(&self, _cx: i32, _cy: i32) -> Option<String> {
        None
    }

    /// Get the category/shape of this algorithm (e.g., "circle", "ellipse", "line", "parabola").
    fn category(&self) -> &'static str;
}
