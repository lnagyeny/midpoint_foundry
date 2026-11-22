//! Independent hexagonal-grid experiment: concentric, gap-filled circle layers.
//!
//! This is a **new** algorithm in its own module. It is modelled on the idea in
//! `algorithms::circle::gap_fill_circle::GapFillCircle` — rasterise a circle at
//! every radius and keep each ring's own identity — but expressed entirely in
//! the existing axial hex coordinate system:
//!
//! - the outline at radius `r` comes from the validated integer midpoint walk
//!   and 12-way integer symmetry in
//!   [`crate::algorithms::hex::midpoint_hex_circle::circle_points`];
//! - "gap filling" means rasterising **every** intermediate radius from 1 up to
//!   a layer's radius and taking the union, exactly as the square-grid
//!   reference accumulates its rings, so no interior cell is skipped;
//! - each layer keeps only cells not claimed by a smaller layer, so a new layer
//!   paints the *newly added band* and never overwrites an earlier one.
//!
//! No floating point is used for coordinates, radii, the decision walk or the
//! symmetry maps. Floating point appears only in the render-only colour helper
//! and in the already-existing hex rendering path.

use std::collections::HashSet;

use crate::algorithms::hex::midpoint_hex_circle::circle_points;
use crate::algorithms::hex::{
    ExperimentGeometry, HexGridExperiment, f_quadratic, hex_to_world, push_hex_fill,
    push_hex_outline,
};
use crate::algorithms::{Algorithm, Point};
use crate::renderer::Vertex;

/// Deterministic colour for layer `index` (0-based).
///
/// Hue advances by 137° per layer (coprime with 360, so consecutive layers are
/// far apart on the colour wheel), and lightness alternates to double the
/// adjacent contrast. There is no randomness: a given index always yields the
/// same RGB triple. This is *presentation* only — never used in rasterisation.
pub fn layer_color(index: usize) -> [f32; 3] {
    let hue = ((index as i32 * 137).rem_euclid(360)) as f32 / 360.0;
    let lightness = if index.is_multiple_of(2) { 0.55 } else { 0.42 };
    hsl_to_rgb(hue, 0.85, lightness)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - 0.5 * c;
    let sector = (h * 6.0) as usize % 6;
    let (r, g, b) = match sector {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [r + m, g + m, b + m]
}

/// Radii of the layers, strictly increasing and clamped to a usable range.
///
/// Returns an empty vector when `min_radius < 1`, `max_radius < min_radius` or
/// `step < 1`; otherwise `min, min+step, …` while `<= max_radius`.
pub fn layer_radii(min_radius: i32, max_radius: i32, step: i32) -> Vec<i32> {
    if min_radius < 1 || max_radius < min_radius || step < 1 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut r = min_radius;
    loop {
        out.push(r);
        if r > max_radius - step {
            break;
        }
        r += step;
    }
    out
}

/// Description of one emitted layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerInfo {
    pub radius: i32,
    pub color: [f32; 3],
    /// Offset of this layer's first cell in `points` / `point_layers`.
    pub start: usize,
    pub len: usize,
}

/// Selectable layered gap-fill circle on the hexagonal grid.
#[derive(Clone)]
pub struct LayeredGapFillCircle {
    pub min_radius: i32,
    pub max_radius: i32,
    pub radius_step: i32,
    /// When true each layer is the filled band up to its radius; when false
    /// only the rasterised outline is painted.
    pub filled: bool,
    pub show_outlines: bool,
    /// Minimum half-extent of the drawn background region.
    pub world_half: i32,

    points: Vec<Point>,
    /// Layer index for every point, aligned with `points`.
    point_layers: Vec<usize>,
    layers: Vec<LayerInfo>,
}

impl Default for LayeredGapFillCircle {
    fn default() -> Self {
        let mut s = Self {
            min_radius: 2,
            max_radius: 10,
            radius_step: 2,
            filled: true,
            show_outlines: true,
            world_half: 12,
            points: Vec::new(),
            point_layers: Vec::new(),
            layers: Vec::new(),
        };
        s.compute();
        s
    }
}

impl LayeredGapFillCircle {
    /// Half-extent used for both the viewport and the background tiling: the
    /// whole outermost layer must always be visible.
    pub fn effective_half(&self) -> i32 {
        self.max_radius.max(self.world_half).max(1)
    }

    #[allow(dead_code)] // research/test API; not called by the binary itself
    pub fn layers(&self) -> &[LayerInfo] {
        &self.layers
    }

    #[allow(dead_code)] // research/test API; not called by the binary itself
    pub fn point_layers(&self) -> &[usize] {
        &self.point_layers
    }

    /// Bounds of a cell list, used by tests to confirm centring and that no
    /// cell lies outside the circle it belongs to.
    #[allow(dead_code)] // research/test API; not called by the binary itself
    pub fn layer_cells(&self, layer: usize) -> impl Iterator<Item = Point> + '_ {
        self.layers
            .get(layer)
            .map(|info| {
                self.points[info.start..info.start + info.len]
                    .iter()
                    .copied()
            })
            .into_iter()
            .flatten()
    }

    /// Build hex-grid geometry: background tiling, one coloured band per layer,
    /// optional cell outlines, and hover/selection highlights.
    pub fn build_geometry(
        &self,
        mouse_cell: Option<(i32, i32)>,
        selected_cell: Option<(i32, i32)>,
    ) -> ExperimentGeometry {
        let mut lines = Vec::<Vertex>::new();
        let mut quads = Vec::<Vertex>::new();

        let grid = HexGridExperiment {
            world_half: self.effective_half(),
            ..HexGridExperiment::default()
        };
        for (x, y) in grid.region_cells() {
            let (cx, cy) = hex_to_world(x, y);
            push_hex_fill(&mut quads, cx, cy, [0.30, 0.30, 0.34, 0.30]);
            push_hex_outline(&mut lines, cx, cy, [0.22, 0.22, 0.26, 1.0]);
        }

        // One band per layer. Cells are disjoint by construction (compute()
        // claims each coordinate once), so nothing is overwritten.
        for layer in &self.layers {
            let [r, g, b] = layer.color;
            for point in &self.points[layer.start..layer.start + layer.len] {
                let (cx, cy) = hex_to_world(point.x, point.y);
                push_hex_fill(&mut quads, cx, cy, [r, g, b, 0.85]);
                if self.show_outlines {
                    push_hex_outline(&mut lines, cx, cy, [0.65 * r, 0.65 * g, 0.65 * b, 1.0]);
                }
            }
        }

        if let Some((hx, hy)) = mouse_cell {
            let (cx, cy) = hex_to_world(hx, hy);
            push_hex_fill(&mut quads, cx, cy, [0.5, 0.7, 1.0, 0.15]);
            push_hex_outline(&mut lines, cx, cy, [0.5, 0.7, 1.0, 0.9]);
        }
        if let Some((sx, sy)) = selected_cell {
            let (cx, cy) = hex_to_world(sx, sy);
            push_hex_fill(&mut quads, cx, cy, [1.0, 1.0, 0.0, 0.45]);
            push_hex_outline(&mut lines, cx, cy, [1.0, 1.0, 0.0, 1.0]);
        }

        ExperimentGeometry {
            line_verts: lines,
            quad_verts: quads,
        }
    }
}

impl Algorithm for LayeredGapFillCircle {
    fn name(&self) -> &str {
        "Layered Gap-Fill Circle (hex)"
    }

    fn category(&self) -> &'static str {
        "hex"
    }

    fn compute(&mut self) {
        self.points.clear();
        self.point_layers.clear();
        self.layers.clear();

        let mut claimed: HashSet<(i32, i32)> = HashSet::new();
        // Union of all integer midpoint outlines from radius 1 upwards. This
        // is the hexagonal analogue of the reference's accumulated rings.
        let mut gap_filled: HashSet<(i32, i32)> = HashSet::new();
        gap_filled.insert((0, 0));
        let mut next_ring = 1;

        for (index, &radius) in layer_radii(self.min_radius, self.max_radius, self.radius_step)
            .iter()
            .enumerate()
        {
            while next_ring <= radius {
                for cell in circle_points(next_ring) {
                    gap_filled.insert((cell.x, cell.y));
                }
                next_ring += 1;
            }

            let mut candidates: Vec<(i32, i32)> = if self.filled {
                gap_filled.iter().copied().collect()
            } else {
                circle_points(radius).iter().map(|p| (p.x, p.y)).collect()
            };
            candidates.sort_unstable();

            let start = self.points.len();
            let mut added = 0;
            for (x, y) in candidates {
                if claimed.insert((x, y)) {
                    self.points.push(Point { x, y });
                    self.point_layers.push(index);
                    added += 1;
                }
            }
            self.layers.push(LayerInfo {
                radius,
                color: layer_color(index),
                start,
                len: added,
            });
        }
    }

    fn points(&self) -> &[Point] {
        &self.points
    }

    fn color(&self) -> [f32; 3] {
        layer_color(0)
    }

    fn point_color_override(&self, i: usize) -> Option<[f32; 3]> {
        Some(layer_color(*self.point_layers.get(i)?))
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.label("Hex layered gap-fill circle — integer midpoint + 12-way symmetry");
        ui.separator();
        changed |= ui
            .add(egui::Slider::new(&mut self.min_radius, 1..=20).text("Minimum radius"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut self.max_radius, 1..=30).text("Maximum radius"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut self.radius_step, 1..=5).text("Layer spacing"))
            .changed();
        changed |= ui.checkbox(&mut self.filled, "Filled bands").changed();
        changed |= ui
            .checkbox(&mut self.show_outlines, "Outline layer cells")
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut self.world_half, 4..=40).text("Minimum region half-extent"))
            .changed();

        if self.max_radius < self.min_radius {
            self.max_radius = self.min_radius;
            changed = true;
        }
        if self.radius_step < 1 {
            self.radius_step = 1;
            changed = true;
        }
        ui.separator();
        ui.label("Each radius adds a new, differently coloured band; earlier bands are kept.");
        changed
    }

    fn viewport(&self) -> Option<(f32, f32, f32)> {
        Some((0.0, 0.0, self.effective_half() as f32 + 1.0))
    }

    fn overlay_geometry(
        &self,
        mouse_cell: Option<(i32, i32)>,
        selected_cell: Option<(i32, i32)>,
    ) -> Option<ExperimentGeometry> {
        Some(self.build_geometry(mouse_cell, selected_cell))
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        let i = self.points.iter().position(|p| p.x == cx && p.y == cy)?;
        let layer = self.point_layers[i];
        let info = self.layers[layer];
        Some(format!(
            "axial ({cx}, {cy})  cube s={}\nlayer {} / radius {}\nF = {}",
            -cx - cy,
            layer + 1,
            info.radius,
            f_quadratic(cx, cy),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FNV-1a of the read-only square-grid reference at the time this module
    /// was written. Guards against accidental edits to that file.
    const REFERENCE_FNV1A: u64 = 0xa5d68642b71c2bca;

    fn fnv1a(bytes: &[u8]) -> u64 {
        let mut h = 0xcbf29ce484222325u64;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    #[test]
    fn layer_radii_are_strictly_increasing() {
        assert_eq!(layer_radii(2, 10, 2), vec![2, 4, 6, 8, 10]);
        assert_eq!(layer_radii(1, 5, 1), vec![1, 2, 3, 4, 5]);
        assert_eq!(layer_radii(3, 10, 4), vec![3, 7]);
        for (min, max, step) in [(1, 20, 1), (2, 30, 3), (5, 12, 2)] {
            let radii = layer_radii(min, max, step);
            assert!(!radii.is_empty());
            for pair in radii.windows(2) {
                assert!(pair[0] < pair[1], "{pair:?}");
            }
            assert!(radii.iter().all(|&r| (min..=max).contains(&r)));
        }
        // Invalid configuration yields no layers rather than a bad radius.
        assert!(layer_radii(0, 10, 1).is_empty());
        assert!(layer_radii(5, 4, 1).is_empty());
        assert!(layer_radii(1, 10, 0).is_empty());
    }

    #[test]
    fn all_layers_share_the_same_origin_centre() {
        let algo = LayeredGapFillCircle::default();
        assert!(!algo.layers().is_empty());
        for layer in 0..algo.layers().len() {
            let cells: Vec<Point> = algo.layer_cells(layer).collect();
            assert!(!cells.is_empty(), "empty layer {layer}");
            let sx: i64 = cells.iter().map(|p| p.x as i64).sum();
            let sy: i64 = cells.iter().map(|p| p.y as i64).sum();
            assert_eq!(sx, 0, "layer {layer} x centroid");
            assert_eq!(sy, 0, "layer {layer} y centroid");
            // 180° symmetry about the origin: integer, no centre offset.
            let set: HashSet<(i32, i32)> = cells.iter().map(|p| (p.x, p.y)).collect();
            for p in &cells {
                assert!(
                    set.contains(&(-p.x, -p.y)),
                    "layer {layer} not centred at origin: ({},{})",
                    p.x,
                    p.y
                );
            }
        }
    }

    #[test]
    fn each_layer_has_a_distinct_contrasting_color() {
        let algo = LayeredGapFillCircle::default();
        let mut seen: Vec<[f32; 3]> = Vec::new();
        for i in 0..32 {
            let color = layer_color(i);
            assert!(color.iter().all(|c| (0.0..=1.0).contains(c)), "color {i}");
            assert!(!seen.contains(&color), "duplicate color at layer {i}");
            seen.push(color);
            if i > 0 {
                let prev = layer_color(i - 1);
                let delta: f32 = color
                    .iter()
                    .zip(prev.iter())
                    .map(|(a, b)| (a - b).abs())
                    .sum();
                assert!(
                    delta > 1.0,
                    "adjacent layers {} / {i} contrast {delta}",
                    i - 1
                );
            }
        }
        for layer in algo.layers() {
            assert_eq!(layer.color, layer_color(layer.radius as usize / 2 - 1));
        }
    }

    #[test]
    fn layer_cells_are_valid_hex_coordinates() {
        let algo = LayeredGapFillCircle::default();
        for (i, p) in algo.points().iter().enumerate() {
            assert!(f_quadratic(p.x, p.y) >= 0);
            // The render transform round trip must recover the same axial cell.
            let (wx, wy) = hex_to_world(p.x, p.y);
            assert_eq!(
                crate::algorithms::hex::world_to_hex(wx, wy),
                (p.x, p.y),
                "point {i}"
            );
            assert_eq!(
                algo.point_color_override(i),
                Some(algo.layers()[algo.point_layers()[i]].color)
            );
        }
    }

    #[test]
    fn no_duplicate_cells_within_a_layer() {
        let algo = LayeredGapFillCircle::default();
        for layer in 0..algo.layers().len() {
            let cells: Vec<(i32, i32)> = algo.layer_cells(layer).map(|p| (p.x, p.y)).collect();
            let unique: HashSet<_> = cells.iter().copied().collect();
            assert_eq!(cells.len(), unique.len(), "duplicates in layer {layer}");
        }
        let all: Vec<(i32, i32)> = algo.points().iter().map(|p| (p.x, p.y)).collect();
        let unique: HashSet<_> = all.iter().copied().collect();
        assert_eq!(all.len(), unique.len(), "duplicate across layers");
    }

    #[test]
    fn filled_bands_do_not_overwrite_earlier_layers() {
        let algo = LayeredGapFillCircle::default();
        let radii: Vec<i32> = algo.layers().iter().map(|l| l.radius).collect();
        assert_eq!(radii, layer_radii(2, 10, 2));

        // Build the independent reference: union of all midpoint outlines up
        // to the outermost radius, plus the origin.
        let mut reference = HashSet::new();
        reference.insert((0, 0));
        for r in 1..=*radii.last().unwrap() {
            for p in circle_points(r) {
                reference.insert((p.x, p.y));
            }
        }

        let mut covered = HashSet::new();
        for (index, layer) in algo.layers().iter().enumerate() {
            let cells: Vec<(i32, i32)> = algo.layer_cells(index).map(|p| (p.x, p.y)).collect();
            for cell in &cells {
                assert!(covered.insert(*cell), "layer {index} overwrote {cell:?}");
                assert!(
                    reference.contains(cell),
                    "layer {index} off-circle {cell:?}"
                );
            }
            // The band is exactly the newly filled region: every cell in the
            // reference at this layer's radius that was not claimed earlier.
            let mut expected = HashSet::new();
            expected.insert((0, 0));
            for r in 1..=layer.radius {
                for p in circle_points(r) {
                    expected.insert((p.x, p.y));
                }
            }
            expected.retain(|c| !covered.contains(c) || cells.contains(c));
            for cell in &expected {
                assert!(
                    cells.contains(cell),
                    "layer {index} missed newly added cell {cell:?}"
                );
            }
            for cell in &cells {
                assert!(
                    expected.contains(cell),
                    "layer {index} emitted cell outside its new band: {cell:?}"
                );
            }
        }
        let emitted: HashSet<(i32, i32)> = algo.points().iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(emitted, reference, "union of bands != gap-filled disk");
    }

    #[test]
    fn outline_only_layers_still_have_disjoint_cells() {
        let algo = LayeredGapFillCircle {
            filled: false,
            ..LayeredGapFillCircle::default()
        };
        assert!(!algo.points().is_empty());
        let all: HashSet<(i32, i32)> = algo.points().iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(all.len(), algo.points().len());
        for layer in algo.layers() {
            assert!(layer.len > 0, "empty outline layer r={}", layer.radius);
        }
    }

    #[test]
    fn geometry_colors_follow_layer_colours() {
        let algo = LayeredGapFillCircle::default();
        let geom = algo.overlay_geometry(Some((1, 0)), Some((0, 1))).unwrap();
        assert_eq!(geom.quad_verts.len() % 18, 0);
        let mut colors = Vec::new();
        for chunk in geom.quad_verts.chunks(18) {
            let v = chunk[0];
            let c = [v.color[0], v.color[1], v.color[2]];
            if !colors.contains(&c) {
                colors.push(c);
            }
        }
        for layer in algo.layers() {
            assert!(colors.contains(&layer.color), "missing layer color");
        }
        // Hover/selection highlights are present as well.
        assert!(!geom.line_verts.is_empty());
    }

    #[test]
    fn reference_gap_fill_circle_source_is_unchanged() {
        const REFERENCE: &str = include_str!("../circle/gap_fill_circle.rs");
        assert_eq!(
            fnv1a(REFERENCE.as_bytes()),
            REFERENCE_FNV1A,
            "read-only square-grid reference was modified"
        );
    }
}
