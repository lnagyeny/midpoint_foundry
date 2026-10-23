//! Hexagonal-grid research module (research-only, isolated from square-grid code).
//!
//! # Status: EXPERIMENT, not an algorithm
//!
//! This module contains **no midpoint algorithm**. It exists to *visually
//! investigate* the geometry of a hexagonal cell grid and to compare candidate
//! boundary definitions before a final midpoint rasterisation rule is adopted.
//! See `docs/research/hexagonal-grid.md` §11 (proposed model v0) and §12 (this
//! experiment).
//!
//! # Coordinate convention (mapping A)
//!
//! Integer `(x, y)` are **axial hex coordinates**. The Cartesian position of a
//! cell centre uses mapping A:
//!
//! ```text
//! X = (√3/2)·x ,   Y = y + x/2
//! ```
//!
//! Under this mapping the six nearest integer neighbours of any cell are
//! exactly the six axial moves at Cartesian distance 1, and the Euclidean
//! circle of radius `r` is the level set of
//!
//! ```text
//! F(x, y) = x² + xy + y² = r²
//! ```
//!
//! These facts are asserted in the `#[cfg(test)]` tests below.
//!
//! # What is drawn
//!
//! - The ideal circle `F = r²` (a true Euclidean circle, because the screen
//!   layout *is* mapping A — see "orientation" below).
//! - Flat-top hexagon cells, each classified by an explicit, documented rule
//!   comparing its centre value `F` to `r²` (see [`CellClass`]).
//!
//! ## Orientation and the render transform
//!
//! The mathematical coordinate system is **mapping A** and is not changed by
//! this experiment: `X = (√3/2)·x`, `Y = y + x/2`, with `F = x²+xy+y²`. Under
//! mapping A the six nearest integer neighbours are at Cartesian distance 1
//! and `F`'s level sets are circles.
//!
//! The lattice produced by mapping A is a **flat-top** hexagonal tiling: the
//! neighbour centre directions are at 30°, 90°, 150°, … (so shared edges are
//! horizontal-ish flat sides), and hexagons with apothem `1/2`
//! (circumradius `R = 1/√3`) tile it edge-to-edge.
//!
//! The *rendering* transform is kept strictly separate from the maths:
//! [`HexGridExperiment`] returns a [`Algorithm::viewport`] override, and the
//! renderer maps the world-space square to a **square screen viewport** with a
//! single uniform scale on both axes. This is what stops the grid from looking
//! diagonally skewed when the window is not square, and keeps circles
//! circular. The axial coordinates, `F`, the neighbours and the radius all
//! keep their exact mathematical meaning.
//!
//! The classification is **not** a rasterised boundary. There is deliberately
//! no claim about "the correct hex midpoint outline": the boundary definition
//! (single closest cell vs. the pair of bracketing cells) is unresolved
//! (`docs/research/hexagonal-grid.md` §11.6, §11.10).

use crate::algorithms::{Algorithm, Point};
use crate::renderer::Vertex;

// ── Pure scalar maths (no rendering, no GPU) ─────────────────────────────────

/// The Euclidean quadratic form under mapping A.
///
/// `F(x, y) = x² + xy + y²`. Level sets of `F` are circles in the Cartesian
/// plane of mapping A.
#[inline]
pub fn f_quadratic(x: i32, y: i32) -> i64 {
    // i64 keeps this exact for any radius the slider can reach.
    let x = x as i64;
    let y = y as i64;
    x * x + x * y + y * y
}

/// Cartesian position of an axial cell centre under mapping A.
#[inline]
pub fn axial_to_cartesian(x: i32, y: i32) -> (f32, f32) {
    (SQRT3_OVER_2 * x as f32, y as f32 + 0.5 * x as f32)
}

/// √3 / 2, the horizontal spacing of the centre lattice.
pub const SQRT3_OVER_2: f32 = 0.866_025_4;

/// Inverse of [`axial_to_cartesian`]: fractional axial coordinates of a
/// Cartesian point under mapping A.
///
/// Solving `X = (√3/2)·x`, `Y = y + x/2` gives
/// `x = X / (√3/2)`, `y = Y − x/2`. The result is fractional; rounding it to
/// the nearest axial cell is [`world_to_axial`].
#[inline]
pub fn cartesian_to_axial_fractional(wx: f32, wy: f32) -> (f32, f32) {
    let x = wx / SQRT3_OVER_2;
    let y = wy - 0.5 * x;
    (x, y)
}

/// Nearest axial cell to a Cartesian point (this is the hex hit test).
///
/// Rounds the fractional axial coordinates with **cube rounding**, which is
/// correct for any hex layout: naive independent rounding of `x` and `y`
/// would pick the wrong cell near a hexagon boundary. The three cube
/// coordinates are `(x, y, z)` with `z = −x − y`; the component with the
/// largest rounding error is recomputed so that `x + y + z = 0` exactly.
#[inline]
pub fn world_to_axial(wx: f32, wy: f32) -> (i32, i32) {
    let (fx, fy) = cartesian_to_axial_fractional(wx, wy);
    let fz = -fx - fy;

    let rx = fx.round();
    let ry = fy.round();
    let rz = fz.round();

    let dx = (rx - fx).abs();
    let dy = (ry - fy).abs();
    let dz = (rz - fz).abs();

    if dx > dy && dx > dz {
        // x had the largest error: recompute it from y, z.
        ((-ry - rz) as i32, ry as i32)
    } else if dy > dz {
        (rx as i32, (-rx - rz) as i32)
    } else {
        (rx as i32, ry as i32)
    }
}

/// The six axial neighbour offsets, in a fixed order (0 … 5).
///
/// These are exactly the six nearest integer neighbours of any cell under
/// mapping A, all at Cartesian distance 1.
pub const NEIGHBOURS: [(i32, i32); 6] = [
    (1, 0),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (0, -1),
    (1, -1),
];

/// Hex metric (cube distance) between two axial cells.
///
/// Part of the research API; exercised by unit tests. Not called by the binary
/// yet, so `dead_code` is expected.
#[allow(dead_code)]
#[inline]
pub fn hex_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    let (dq, dr) = (a.0 - b.0, a.1 - b.1);
    let ds = -dq - dr;
    (dq.abs() + dr.abs() + ds.abs()) / 2
}

/// The 60° lattice rotation about the origin, `(x, y) → (x + y, −x)`.
///
/// Preserves `F` (tested). Applying it six times returns to the origin cell.
/// Part of the research API; exercised by unit tests. Not called by the binary
/// yet, so `dead_code` is expected.
#[allow(dead_code)]
#[inline]
pub fn rotate60(x: i32, y: i32) -> (i32, i32) {
    (x + y, -x)
}

// ── Explicit, documented classification (NOT a boundary) ─────────────────────

/// Geometric relationship of a cell centre to the ideal circle `F = r²`.
///
/// This is an **inspection aid**, not a rasterised boundary. It answers only
/// "where is this cell centre relative to the circle?" — it does **not** say
/// whether the cell belongs to a hex midpoint outline, because that boundary
/// definition is still unresolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellClass {
    /// `F < r²`: the centre is strictly inside the circle.
    Inside,
    /// `F == r²`: the centre lies exactly on the circle. Always non-empty, since
    /// `F(0,r) = F(r,0) = r²` for every `r ≥ 1`.
    OnCircle,
    /// `0 < F − r² ≤ r`: just outside, within roughly half a lattice step.
    NearOutside,
    /// `F − r² > r`: clearly outside the band of interest.
    FarOutside,
}

impl CellClass {
    /// Classify a cell centre at radius `r`. `r` must be ≥ 0.
    ///
    /// The "near" band threshold `r` is a **display choice** chosen so that the
    /// band is roughly half a lattice step thick at radius `r`; it is not
    /// derived from any midpoint rule. (One lattice step changes `F` by about
    /// `2r`, so a band of width `r` in `F` is about half a step.)
    pub fn of(x: i32, y: i32, r: i32) -> CellClass {
        let f = f_quadratic(x, y);
        let r2 = r as i64 * r as i64;
        match f.cmp(&r2) {
            std::cmp::Ordering::Less => CellClass::Inside,
            std::cmp::Ordering::Equal => CellClass::OnCircle,
            std::cmp::Ordering::Greater => {
                if f - r2 <= r as i64 {
                    CellClass::NearOutside
                } else {
                    CellClass::FarOutside
                }
            }
        }
    }

    /// Display colour (RGBA) for this class. Translucent so overlaps are safe.
    pub fn color(self) -> [f32; 4] {
        match self {
            CellClass::Inside => [0.25, 0.55, 1.0, 0.35], // blue-ish fill
            CellClass::OnCircle => [1.0, 1.0, 1.0, 0.85], // white
            CellClass::NearOutside => [1.0, 0.75, 0.15, 0.55], // amber
            CellClass::FarOutside => [0.35, 0.35, 0.40, 0.18], // dim grey fill
        }
    }
}

// ── Hexagon geometry (pure, testable) ────────────────────────────────────────

/// Circumradius of the flat-top hexagons.
///
/// The nearest-neighbour distance of the centre lattice is 1, so adjacent
/// hexagons must share edges when their apothem is `1/2`. For a flat-top
/// hexagon the apothem is `(√3/2)·R`, hence `R = 1/√3 = √3/3`.
pub const HEX_CIRCUMRADIUS: f32 = 0.577_350_26; // √3 / 3

/// The six vertices of the flat-top hexagon centred at `(cx, cy)`.
///
/// Vertex angles are `0°, 60°, …, 300°`, giving flat top and bottom sides.
/// Returned in counter-clockwise order, suitable for a triangle fan.
pub fn hexagon_vertices(cx: f32, cy: f32) -> [(f32, f32); 6] {
    let mut out = [(0.0f32, 0.0f32); 6];
    let mut i = 0;
    while i < 6 {
        let angle = std::f32::consts::FRAC_PI_3 * i as f32; // 60° steps
        out[i] = (
            cx + HEX_CIRCUMRADIUS * angle.cos(),
            cy + HEX_CIRCUMRADIUS * angle.sin(),
        );
        i += 1;
    }
    out
}

/// Push a filled hexagon as a triangle fan (6 triangles = 18 vertices; the
/// centre vertex is repeated, which avoids internal seams and needs no indices).
#[inline]
pub fn push_hex_fill(buf: &mut Vec<Vertex>, cx: f32, cy: f32, color: [f32; 4]) {
    let v = hexagon_vertices(cx, cy);
    let c = Vertex::new(cx, cy, color[0], color[1], color[2], color[3]);
    for i in 0..6 {
        let a = v[i];
        let b = v[(i + 1) % 6];
        buf.push(c);
        buf.push(Vertex::new(a.0, a.1, color[0], color[1], color[2], color[3]));
        buf.push(Vertex::new(b.0, b.1, color[0], color[1], color[2], color[3]));
    }
}

/// Push the six outline edges of the flat-top hexagon at `(cx, cy)`.
#[inline]
pub fn push_hex_outline(buf: &mut Vec<Vertex>, cx: f32, cy: f32, color: [f32; 4]) {
    let v = hexagon_vertices(cx, cy);
    for i in 0..6 {
        let a = v[i];
        let b = v[(i + 1) % 6];
        buf.push(Vertex::new(a.0, a.1, color[0], color[1], color[2], color[3]));
        buf.push(Vertex::new(b.0, b.1, color[0], color[1], color[2], color[3]));
    }
}

// ── Self-contained geometry output for the experiment ────────────────────────

/// Complete geometry produced by a research experiment.
///
/// Returned through [`Algorithm::overlay_geometry`] so the experiment can draw
/// arbitrary polygons (hexagons) without touching the square-grid path in
/// `geometry.rs`.
#[derive(Default, Clone)]
pub struct ExperimentGeometry {
    pub line_verts: Vec<Vertex>,
    pub quad_verts: Vec<Vertex>,
}

// ── The experiment algorithm ─────────────────────────────────────────────────

/// Research-only visualisation of a hexagonal cell grid and the ideal circle
/// `F(x, y) = r²`.
///
/// Selecting this in the UI replaces the square-grid view **for this algorithm
/// only**; every existing square-grid algorithm renders exactly as before
/// (their `overlay_geometry` defaults to `None`).
#[derive(Clone)]
pub struct HexGridExperiment {
    /// Radius of the ideal circle `F = r²`.
    pub radius: i32,
    /// Half-extent of the drawn region, in **world-space** units.
    ///
    /// The cells are generated to cover the world-space square
    /// `[−world_half, +world_half]²` (with margin); the visible square is one
    /// unit larger because [`HexGridExperiment::viewport`] renders
    /// `[−(world_half+1), +(world_half+1)]²`, so edge hexagons are clipped by
    /// the viewport rather than ending exactly on the boundary. This is what
    /// makes the tiling fill a square area rather than a sheared rhombus in
    /// axial space.
    pub world_half: i32,
    /// Draw the ideal-circle outline.
    pub show_circle: bool,
    /// Show cell classification fills.
    pub show_classes: bool,
    /// Highlight the six neighbours of the origin (adjacency demonstration).
    pub show_neighbours: bool,
}

impl Default for HexGridExperiment {
    fn default() -> Self {
        Self {
            radius: 5,
            world_half: 12,
            show_circle: true,
            show_classes: true,
            show_neighbours: true,
        }
    }
}

impl HexGridExperiment {
    /// Axial cells whose hexagons overlap the world square `[−H, H]²`.
    ///
    /// For each integer axial `x`, the world `X` is `(√3/2)·x`, so
    /// `|x| ≤ ⌈H/(√3/2)⌉`. For each such `x`, `Y = y + x/2`, so `y` ranges
    /// over `[−H − x/2, H − x/2]`, expanded by one cell so hexagons that
    /// straddle the edge are still emitted (the viewport clips them).
    /// Row-major (`y` outer, `x` inner).
    pub fn region_cells(&self) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        let h = self.world_half.max(0) as f32 + HEX_CIRCUMRADIUS + 1.0;
        let x_max = (h / SQRT3_OVER_2).ceil() as i32 + 1;
        for x in -x_max..=x_max {
            let yc = -0.5 * x as f32;
            let y_min = (yc - h).floor() as i32 - 1;
            let y_max = (yc + h).ceil() as i32 + 1;
            for y in y_min..=y_max {
                out.push((x, y));
            }
        }
        out
    }

    /// Build the full experimental geometry. Pure with respect to the GPU:
    /// this only produces `Vertex` values.
    ///
    /// `mouse_cell` and `selected_cell` are axial cells (already converted by
    /// `input.rs`); they are drawn as hover/selection highlights in the hex
    /// coordinate system.
    pub fn build_geometry(
        &self,
        mouse_cell: Option<(i32, i32)>,
        selected_cell: Option<(i32, i32)>,
    ) -> ExperimentGeometry {
        let mut lines = Vec::<Vertex>::new();
        let mut quads = Vec::<Vertex>::new();

        let r = self.radius.max(0);
        // Computed once and reused for both the fill and outline passes.
        let cells = self.region_cells();

        // 1. Cell fills, classified by their relationship to the circle.
        for &(x, y) in &cells {
            let (cx, cy) = axial_to_cartesian(x, y);
            let color = if self.show_classes {
                CellClass::of(x, y, r).color()
            } else {
                [0.30, 0.30, 0.34, 0.30]
            };
            push_hex_fill(&mut quads, cx, cy, color);
        }

        // 2. Hover cell (soft blue, same intent as the square-grid hover row).
        if let Some((hx, hy)) = mouse_cell {
            let (cx, cy) = axial_to_cartesian(hx, hy);
            push_hex_fill(&mut quads, cx, cy, [0.5, 0.7, 1.0, 0.15]);
        }

        // 3. Selected cell (yellow, same intent as the square-grid selection).
        if let Some((sx, sy)) = selected_cell {
            let (cx, cy) = axial_to_cartesian(sx, sy);
            push_hex_fill(&mut quads, cx, cy, [1.0, 1.0, 0.0, 0.45]);
        }

        // 4. Cell outlines (thin grey) on top of the fills.
        for &(x, y) in &cells {
            let (cx, cy) = axial_to_cartesian(x, y);
            push_hex_outline(&mut lines, cx, cy, [0.22, 0.22, 0.26, 1.0]);
        }

        // 5. Six neighbours of the origin (bright green outlines).
        if self.show_neighbours {
            for (dx, dy) in NEIGHBOURS {
                let (cx, cy) = axial_to_cartesian(dx, dy);
                push_hex_outline(&mut lines, cx, cy, [0.2, 1.0, 0.4, 0.9]);
            }
            // Origin marker.
            let (cx, cy) = axial_to_cartesian(0, 0);
            push_hex_outline(&mut lines, cx, cy, [1.0, 1.0, 1.0, 0.9]);
        }

        // 6. Highlight outlines for hover and selection cells.
        if let Some((hx, hy)) = mouse_cell {
            let (cx, cy) = axial_to_cartesian(hx, hy);
            push_hex_outline(&mut lines, cx, cy, [0.5, 0.7, 1.0, 0.9]);
        }
        if let Some((sx, sy)) = selected_cell {
            let (cx, cy) = axial_to_cartesian(sx, sy);
            push_hex_outline(&mut lines, cx, cy, [1.0, 1.0, 0.0, 1.0]);
        }

        // 7. The ideal circle F = r², drawn as a true Euclidean circle.
        if self.show_circle && r > 0 {
            let segments = 180usize;
            let rf = r as f32;
            for i in 0..segments {
                let a0 = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
                let a1 = 2.0 * std::f32::consts::PI * (i + 1) as f32 / segments as f32;
                lines.push(Vertex::new(
                    rf * a0.cos(),
                    rf * a0.sin(),
                    1.0,
                    1.0,
                    1.0,
                    1.0,
                ));
                lines.push(Vertex::new(
                    rf * a1.cos(),
                    rf * a1.sin(),
                    1.0,
                    1.0,
                    1.0,
                    1.0,
                ));
            }
        }

        ExperimentGeometry {
            line_verts: lines,
            quad_verts: quads,
        }
    }
}

impl Algorithm for HexGridExperiment {
    fn name(&self) -> &str {
        "Hex grid experiment"
    }

    fn compute(&mut self) {
        // Nothing to precompute: geometry is built on demand in
        // `overlay_geometry`. Kept because the trait requires it.
    }

    fn points(&self) -> &[Point] {
        // The experiment draws its own geometry; it exposes no `Point` list.
        &[]
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.label("Research experiment — no midpoint algorithm");
        ui.separator();
        changed |= ui
            .add(egui::Slider::new(&mut self.radius, 1..=20).text("Radius r (F = r²)"))
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.world_half, 4..=40)
                    .text("Region half-extent (world units)"),
            )
            .changed();
        ui.separator();
        changed |= ui
            .checkbox(&mut self.show_circle, "Ideal circle F = r²")
            .changed();
        changed |= ui
            .checkbox(&mut self.show_classes, "Classify cells by centre F vs r²")
            .changed();
        changed |= ui
            .checkbox(&mut self.show_neighbours, "Highlight 6 neighbours of origin")
            .changed();
        ui.separator();
        ui.label("Legend:");
        ui.colored_label(
            egui::Color32::from_rgb(64, 140, 255),
            "inside (F < r²)",
        );
        ui.colored_label(egui::Color32::WHITE, "on circle (F = r²)");
        ui.colored_label(egui::Color32::from_rgb(255, 191, 38), "near outside (0 < F−r² ≤ r)");
        ui.colored_label(egui::Color32::GRAY, "far outside (F−r² > r)");
        ui.separator();
        ui.label(
            "Classification is an inspection aid only. It is NOT a midpoint \
             boundary; the boundary definition is unresolved.",
        );
        changed
    }

    fn category(&self) -> &'static str {
        "hex"
    }

    fn overlay_geometry(
        &self,
        mouse_cell: Option<(i32, i32)>,
        selected_cell: Option<(i32, i32)>,
    ) -> Option<ExperimentGeometry> {
        Some(self.build_geometry(mouse_cell, selected_cell))
    }

    /// Experimental viewport: world `[−H, +H]²` → a square screen viewport.
    ///
    /// `H = world_half + 1` gives a one-cell margin so edge hexagons are
    /// clipped by the viewport rather than ending exactly on the boundary.
    /// The circle (radius `r`) is always comfortably inside for the default
    /// settings; the margin does not change the mathematical coordinates.
    fn viewport(&self) -> Option<(f32, f32, f32)> {
        Some((0.0, 0.0, self.world_half as f32 + 1.0))
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        let f = f_quadratic(cx, cy);
        let r2 = self.radius as i64 * self.radius as i64;
        let class = CellClass::of(cx, cy, self.radius);
        let (ax, ay) = axial_to_cartesian(cx, cy);
        Some(format!(
            "axial ({cx}, {cy})  cube s={}\nF = {f}\nr² = {r2}\nclass = {class:?}\ncart = ({ax:.3}, {ay:.3})",
            -cx - cy
        ))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Mapping A: the six nearest integer neighbours of a cell are exactly the
    /// axial moves, all at Cartesian distance 1.
    #[test]
    fn neighbours_are_at_unit_distance() {
        for (dx, dy) in NEIGHBOURS {
            let (nx, ny) = axial_to_cartesian(dx, dy);
            let d = (nx * nx + ny * ny).sqrt();
            assert!(
                (d - 1.0).abs() < 1e-5,
                "neighbour ({dx},{dy}) distance {d} != 1"
            );
        }
    }

    /// The neighbour set is closed: every neighbour of a cell is also a cell
    /// reachable by one of the six offsets, and all six are distinct.
    #[test]
    fn neighbour_offsets_are_distinct_and_hex_distance_one() {
        let origin = (0, 0);
        let mut seen = std::collections::HashSet::new();
        for (dx, dy) in NEIGHBOURS {
            assert!(seen.insert((dx, dy)), "duplicate neighbour ({dx},{dy})");
            assert_eq!(hex_distance((dx, dy), origin), 1);
        }
        assert_eq!(seen.len(), 6);
    }

    /// The quadratic-form identity: `X² + Y² == x² + xy + y²` under mapping A,
    /// to floating tolerance, and `f_quadratic` is exact in integers.
    #[test]
    fn quadratic_form_identity_holds() {
        for x in -12..=12 {
            for y in -12..=12 {
                let (cx, cy) = axial_to_cartesian(x, y);
                let cart = cx * cx + cy * cy;
                let quad = f_quadratic(x, y) as f32;
                assert!(
                    (cart - quad).abs() < 1e-3,
                    "({x},{y}): X²+Y²={cart} vs x²+xy+y²={quad}"
                );
            }
        }
    }

    /// Integer quadratic form matches the naive polynomial exactly.
    #[test]
    fn f_quadratic_matches_naive() {
        for x in -50..=50 {
            for y in -50..=50 {
                let expected = (x * x + x * y + y * y) as i64;
                assert_eq!(f_quadratic(x, y), expected, "at ({x},{y})");
            }
        }
    }

    /// The 60° rotation preserves `F` and has order 6.
    #[test]
    fn rotate60_preserves_f_and_has_order_six() {
        for x in -8..=8 {
            for y in -8..=8 {
                let (rx, ry) = rotate60(x, y);
                assert_eq!(
                    f_quadratic(rx, ry),
                    f_quadratic(x, y),
                    "rotation changed F at ({x},{y})"
                );
            }
        }
        let (mut x, mut y) = (3, -2);
        for _ in 0..6 {
            let (nx, ny) = rotate60(x, y);
            x = nx;
            y = ny;
        }
        assert_eq!((x, y), (3, -2), "rotation was not order 6");
    }

    /// Flat-top hexagon geometry: the six vertices are at the circumradius and
    /// edge-to-edge tiling works because the apothem equals half the
    /// nearest-neighbour distance (0.5).
    #[test]
    fn hexagon_geometry_is_correct() {
        let v = hexagon_vertices(0.0, 0.0);
        for (i, (vx, vy)) in v.iter().enumerate() {
            let rr = (vx * vx + vy * vy).sqrt();
            assert!(
                (rr - HEX_CIRCUMRADIUS).abs() < 1e-5,
                "vertex {i} radius {rr}"
            );
        }
        // Flat-top: vertex 0 points along +X, so top and bottom edges are flat.
        assert!((v[0].0 - HEX_CIRCUMRADIUS).abs() < 1e-5);
        assert!(v[0].1.abs() < 1e-5);
        // Apothem = R * cos(30°) = 0.57735 * 0.86603 ≈ 0.5.
        let apothem = HEX_CIRCUMRADIUS * (std::f32::consts::FRAC_PI_6).cos();
        assert!(
            (apothem - 0.5).abs() < 1e-4,
            "apothem {apothem} != 0.5 (hexagons would not tile)"
        );
    }

    /// Adjacent hexagons share an edge: the shared edge midpoint is equidistant
    /// (apothem) from both centres.
    #[test]
    fn adjacent_hexagons_share_edges() {
        let (cx0, cy0) = axial_to_cartesian(0, 0);
        for (dx, dy) in NEIGHBOURS {
            let (cx1, cy1) = axial_to_cartesian(dx, dy);
            let mx = 0.5 * (cx0 + cx1);
            let my = 0.5 * (cy0 + cy1);
            let d0 = ((mx - cx0).powi(2) + (my - cy0).powi(2)).sqrt();
            let d1 = ((mx - cx1).powi(2) + (my - cy1).powi(2)).sqrt();
            assert!(
                (d0 - 0.5).abs() < 1e-4 && (d1 - 0.5).abs() < 1e-4,
                "shared edge midpoint not at apothem: d0={d0}, d1={d1}"
            );
        }
    }

    /// Classification boundaries are exactly as documented, including the
    /// `NearOutside` band (`0 < F − r² ≤ r`), which is the display-choice
    /// branch most likely to be wrong and so must be asserted explicitly.
    #[test]
    fn classification_boundaries() {
        let r = 5;
        // (0,0): F=0 < 25 -> Inside.
        assert_eq!(CellClass::of(0, 0, r), CellClass::Inside);
        // (3,4): F = 9+12+16 = 37 > 25, 37−25 = 12 > 5 -> FarOutside.
        assert_eq!(CellClass::of(3, 4, r), CellClass::FarOutside);
        // (2,4): F = 4+8+16 = 28, 28−25 = 3, 0 < 3 ≤ 5 -> NearOutside.
        assert_eq!(CellClass::of(2, 4, r), CellClass::NearOutside);
        // (4,2): F = 16+8+4 = 28, same by symmetry.
        assert_eq!(CellClass::of(4, 2, r), CellClass::NearOutside);
        // (0,5),(5,0): F = 25 = r² -> OnCircle.
        assert_eq!(CellClass::of(0, 5, r), CellClass::OnCircle);
        assert_eq!(CellClass::of(5, 0, r), CellClass::OnCircle);
        // (1,5): F = 31, 31−25 = 6 > 5 -> FarOutside (just past the band).
        assert_eq!(CellClass::of(1, 5, r), CellClass::FarOutside);
    }

    /// Every `OnCircle` cell has `F == r²` exactly, and the branch is non-empty
    /// for every `r ≥ 1` because `(0,r)` and `(r,0)` always qualify.
    #[test]
    fn on_circle_cells_have_exact_f() {
        for r in 1..=10 {
            assert_eq!(CellClass::of(0, r, r), CellClass::OnCircle);
            assert_eq!(CellClass::of(r, 0, r), CellClass::OnCircle);
            for x in -r..=r {
                for y in -r..=r {
                    if CellClass::of(x, y, r) == CellClass::OnCircle {
                        assert_eq!(f_quadratic(x, y), r as i64 * r as i64);
                    }
                }
            }
        }
    }

    /// Geometry builder produces consistent vertex counts (sanity check on the
    /// fan/outline arithmetic: 6 triangles per hex = 18 fill verts, 6 edges per
    /// hex = 12 outline verts), with no hover/selection highlights.
    #[test]
    fn geometry_vertex_counts() {
        let exp = HexGridExperiment {
            radius: 3,
            world_half: 2,
            show_circle: true,
            show_classes: true,
            show_neighbours: true,
        };
        let cells = exp.region_cells().len();
        assert!(cells > 9, "region should cover the world square, got {cells}");
        let g = exp.build_geometry(None, None);
        // Fills: 6 triangles * 3 verts = 18 verts per cell (no highlights).
        assert_eq!(g.quad_verts.len(), cells * 18);
        // Outlines: 6 edges * 2 verts = 12 per cell, plus 6 neighbours and the
        // origin highlight (7 * 12), plus the circle (180 segments * 2 verts).
        let expected_lines = cells * 12 + 7 * 12 + 180 * 2;
        assert_eq!(g.line_verts.len(), expected_lines);
    }

    /// Hover and selection highlights add exactly one fill (18 verts) and one
    /// outline (12 verts) each, in the hex coordinate system.
    #[test]
    fn hover_and_selection_add_highlights() {
        let exp = HexGridExperiment {
            radius: 3,
            world_half: 2,
            show_circle: false,
            show_classes: true,
            show_neighbours: false,
        };
        let cells = exp.region_cells().len();
        let base = exp.build_geometry(None, None);
        let hovered = exp.build_geometry(Some((1, 0)), None);
        let selected = exp.build_geometry(None, Some((0, 1)));
        let both = exp.build_geometry(Some((1, 0)), Some((0, 1)));

        assert_eq!(hovered.quad_verts.len(), base.quad_verts.len() + 18);
        assert_eq!(hovered.line_verts.len(), base.line_verts.len() + 12);
        assert_eq!(selected.quad_verts.len(), base.quad_verts.len() + 18);
        assert_eq!(selected.line_verts.len(), base.line_verts.len() + 12);
        assert_eq!(both.quad_verts.len(), cells * 18 + 2 * 18);
        assert_eq!(both.line_verts.len(), cells * 12 + 2 * 12);
    }

    /// The emitted fill vertices for a single cell really are a closed hexagon
    /// centred on that cell's Cartesian position (renderer-output-level check).
    #[test]
    fn emitted_fill_forms_closed_hexagon() {
        let exp = HexGridExperiment {
            radius: 3,
            world_half: 0, // only the origin cell (plus its one-cell margin)
            show_circle: false,
            show_classes: true,
            show_neighbours: false,
        };
        let g = exp.build_geometry(None, None);
        // Find the origin cell's fan: all six fan centres equal (0, 0).
        let (ox, oy) = axial_to_cartesian(0, 0);
        assert_eq!(g.quad_verts.len() % 18, 0);
        let mut found_origin = false;
        for chunk in g.quad_verts.chunks(18) {
            let centres: Vec<_> = chunk.iter().step_by(3).map(|v| (v.pos[0], v.pos[1])).collect();
            assert_eq!(centres.len(), 6);
            let is_origin = centres
                .iter()
                .all(|(px, py)| (px - ox).abs() < 1e-5 && (py - oy).abs() < 1e-5);
            if !is_origin {
                continue;
            }
            found_origin = true;
            // Every non-centre fill vertex is a radius-`R` hexagon vertex.
            for (i, v) in chunk.iter().enumerate() {
                if i % 3 == 0 {
                    continue;
                }
                let d = ((v.pos[0] - ox).powi(2) + (v.pos[1] - oy).powi(2)).sqrt();
                assert!(
                    (d - HEX_CIRCUMRADIUS).abs() < 1e-5,
                    "fill vertex {i} at radius {d}"
                );
            }
        }
        assert!(found_origin, "origin cell fill was not emitted");
    }

    /// The built circle has the requested Euclidean radius.
    #[test]
    fn circle_outline_has_requested_radius() {
        let exp = HexGridExperiment {
            radius: 7,
            world_half: 2,
            show_circle: true,
            show_classes: false,
            show_neighbours: false,
        };
        let g = exp.build_geometry(None, None);
        // Last 360 vertices are the circle (180 segments * 2).
        let circle = &g.line_verts[g.line_verts.len() - 360..];
        for v in circle {
            let d = (v.pos[0] * v.pos[0] + v.pos[1] * v.pos[1]).sqrt();
            assert!((d - 7.0).abs() < 1e-3, "circle vertex radius {d} != 7");
        }
    }

    /// `axial_to_cartesian` and `cartesian_to_axial_fractional` are exact
    /// inverses on cell centres.
    #[test]
    fn cartesian_axial_round_trip() {
        for x in -20..=20 {
            for y in -20..=20 {
                let (wx, wy) = axial_to_cartesian(x, y);
                let (fx, fy) = cartesian_to_axial_fractional(wx, wy);
                assert!(
                    (fx - x as f32).abs() < 1e-3 && (fy - y as f32).abs() < 1e-3,
                    "round trip ({x},{y}) -> ({fx},{fy})"
                );
            }
        }
    }

    /// `world_to_axial` returns the nearest cell: every cell centre maps back to
    /// its own cell, and points slightly inside a cell also map to it.
    #[test]
    fn world_to_axial_recovers_cell() {
        for x in -10..=10 {
            for y in -10..=10 {
                let (wx, wy) = axial_to_cartesian(x, y);
                assert_eq!(world_to_axial(wx, wy), (x, y), "centre ({x},{y})");
                // A small offset well inside the hexagon (< apothem 0.5).
                for (dx, dy) in [(0.2, 0.0), (0.0, 0.2), (-0.15, 0.1), (0.1, -0.2)] {
                    assert_eq!(
                        world_to_axial(wx + dx, wy + dy),
                        (x, y),
                        "near-centre ({x},{y}) offset ({dx},{dy})"
                    );
                }
            }
        }
    }

    /// The hit test agrees with the geometric definition: a point is assigned
    /// to the nearest cell centre among all candidates. Checked against brute
    /// force over a neighbourhood.
    #[test]
    fn world_to_axial_matches_nearest_centre() {
        for x in -6..=6 {
            for y in -6..=6 {
                let (wx, wy) = axial_to_cartesian(x, y);
                for (dx, dy) in [(0.3, 0.3), (-0.3, 0.2), (0.25, -0.35)] {
                    let (px, py) = (wx + dx, wy + dy);
                    let got = world_to_axial(px, py);
                    let got_d = {
                        let (gx, gy) = axial_to_cartesian(got.0, got.1);
                        (gx - px).powi(2) + (gy - py).powi(2)
                    };
                    // Brute force over a 5x5 candidate window around the cell.
                    for cx in (x - 2)..=(x + 2) {
                        for cy in (y - 2)..=(y + 2) {
                            let (ccx, ccy) = axial_to_cartesian(cx, cy);
                            let d = (ccx - px).powi(2) + (ccy - py).powi(2);
                            assert!(
                                got_d <= d + 1e-4,
                                "({px:.3},{py:.3}) chose {got:?} (d={got_d}) over ({cx},{cy}) (d={d})"
                            );
                        }
                    }
                }
            }
        }
    }

    /// Reference point-in-hexagon test via the six edge normals.
    ///
    /// A point is inside the regular flat-top hexagon iff its signed distance
    /// to every edge line is at most the apothem (0.5). The edge normals point
    /// at 30° + 60k (the six neighbour directions of mapping A).
    fn inside_hexagon(px: f32, py: f32, cx: f32, cy: f32) -> bool {
        const APOTHEM: f32 = 0.5;
        for k in 0..6 {
            let ang = std::f32::consts::FRAC_PI_6 + k as f32 * std::f32::consts::FRAC_PI_3;
            let (nx, ny) = (ang.cos(), ang.sin());
            if (px - cx) * nx + (py - cy) * ny > APOTHEM + 1e-4 {
                return false;
            }
        }
        true
    }

    /// The drawing region must **fully cover** the world square `[−H, H]²`:
    /// every sample point's nearest axial cell is present in `region_cells()`.
    /// This is exhaustive in the discrete domain (nearest-cell assignment is
    /// piecewise constant), so a missing cell cannot slip between samples.
    /// The reference is the hit test `world_to_axial`, not the rendering.
    #[test]
    fn region_covers_world_square() {
        for world_half in [2, 5, 12, 20] {
            let exp = HexGridExperiment {
                world_half,
                ..HexGridExperiment::default()
            };
            let cells: std::collections::HashSet<(i32, i32)> =
                exp.region_cells().into_iter().collect();
            let h = world_half as f32;
            let n = 40; // 41×41 samples per square
            for i in 0..=n {
                for j in 0..=n {
                    let px = -h + 2.0 * h * i as f32 / n as f32;
                    let py = -h + 2.0 * h * j as f32 / n as f32;
                    let cell = world_to_axial(px, py);
                    assert!(
                        cells.contains(&cell),
                        "world_half={world_half}: point ({px},{py}) -> nearest cell {cell:?} is outside the drawn region"
                    );
                    // The covering hexagon must actually contain the point
                    // (independent geometric reference).
                    let (cx, cy) = axial_to_cartesian(cell.0, cell.1);
                    assert!(
                        inside_hexagon(px, py, cx, cy),
                        "world_half={world_half}: point ({px},{py}) not inside cell {cell:?}"
                    );
                }
            }
        }
    }

    /// `region_cells` emits no duplicates (programmatic duplicate detection).
    #[test]
    fn region_cells_have_no_duplicates() {
        let exp = HexGridExperiment {
            world_half: 12,
            ..HexGridExperiment::default()
        };
        let cells = exp.region_cells();
        let unique: std::collections::HashSet<(i32, i32)> = cells.iter().copied().collect();
        assert_eq!(cells.len(), unique.len(), "region_cells emitted duplicates");
    }

    /// Every axial cell whose centre lies at least one lattice step inside the
    /// world square `[−H, H]²` must have **all six** neighbours also present in
    /// `region_cells()`. This guards adjacency completeness of the drawn
    /// region against the world-space reference (the research spec §7, §11.8
    /// adjacency-invariance requirement), and is not tautological: the region
    /// is deliberately padded, so a premature cut-off would be caught.
    #[test]
    fn region_is_closed_under_neighbour_moves_in_interior() {
        let exp = HexGridExperiment {
            world_half: 12,
            ..HexGridExperiment::default()
        };
        let cells: std::collections::HashSet<(i32, i32)> =
            exp.region_cells().into_iter().collect();
        // One hexagonal lattice step is a Unit distance; require a full step of
        // margin so "inside" is unambiguous.
        let inner = exp.world_half as f32 - 1.0;
        let mut checked = 0;
        for &(x, y) in &cells {
            let (cx, cy) = axial_to_cartesian(x, y);
            let (ax, ay) = (cx.abs(), cy.abs());
            // Corners of the hexagon reach R beyond the centre.
            if ax + HEX_CIRCUMRADIUS > inner || ay + HEX_CIRCUMRADIUS > inner {
                continue;
            }
            for (dx, dy) in NEIGHBOURS {
                assert!(
                    cells.contains(&(x + dx, y + dy)),
                    "cell ({x},{y}) at ({cx:.2},{cy:.2}) inside the square lacks neighbour ({},{})",
                    x + dx,
                    y + dy
                );
                checked += 1;
            }
        }
        assert!(checked > 0, "no interior cells were checked");
    }
}
