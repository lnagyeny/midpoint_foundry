use winit::event::{ElementState, MouseButton};

use crate::algorithms::hex;

/// True when the selected algorithm is the hex-grid research experiment.
///
/// Today "has a viewport override" is equivalent to "is the hex experiment",
/// because `HexGridExperiment` is the only `Algorithm` that overrides
/// [`crate::algorithms::Algorithm::viewport`]. The helper names this coupling
/// explicitly so that adding a future non-hex viewport override does not
/// silently route its input through the hex hit test.
#[inline]
fn is_hex_experiment(viewport: Option<(f32, f32, f32)>) -> bool {
    viewport.is_some()
}

// ── Input State ──────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct InputState {
    pub mouse_grid: Option<(f32, f32)>,
    pub selected_cell: Option<(i32, i32)>,
    pub last_mouse_down: bool,
}

// ── Screen ↔ grid-space transform ────────────────────────────────────────────

/// The world-space rectangle actually shown on screen, plus the screen-space
/// rectangle it is drawn into. This mirrors exactly what `Renderer::render`
/// does, so mouse coordinates can be inverted consistently.
///
/// * Default (no viewport override): world `[-half, half]²` → the full grid
///   area rectangle `[0, grid_area_w] × [0, height]`. This is the original
///   square-grid mapping and is **unchanged**.
/// * Override (`Some((cx, cy, half))`): world `[cx−half, cx+half]²` → a
///   **square** screen rectangle of side `min(grid_area_w, height)` centred
///   horizontally in the grid area (matching `Renderer::render`).
pub struct ScreenMap {
    world_cx: f32,
    world_cy: f32,
    world_half: f32,
    /// Screen-space viewport rectangle: (x, y_top, w, h) in physical pixels.
    vp_x: f32,
    vp_y: f32,
    vp_w: f32,
    vp_h: f32,
}

impl ScreenMap {
    pub fn resolve(
        renderer_width: u32,
        renderer_height: u32,
        grid_size: i32,
        viewport: Option<(f32, f32, f32)>,
    ) -> ScreenMap {
        const UI_W: f32 = 350.0; // right-panel pixel width
        let grid_area_w = renderer_width as f32 - UI_W;

        match viewport {
            Some((cx, cy, half)) if half > 0.0 => {
                let side = grid_area_w.min(renderer_height as f32);
                let x = (grid_area_w - side) * 0.5;
                ScreenMap {
                    world_cx: cx,
                    world_cy: cy,
                    world_half: half,
                    vp_x: x,
                    vp_y: 0.0,
                    vp_w: side,
                    vp_h: side,
                }
            }
            _ => {
                let half = grid_size as f32 / 2.0;
                ScreenMap {
                    world_cx: 0.0,
                    world_cy: 0.0,
                    world_half: half,
                    vp_x: 0.0,
                    vp_y: 0.0,
                    vp_w: grid_area_w,
                    vp_h: renderer_height as f32,
                }
            }
        }
    }

    /// Screen pixel → world-space grid coordinate. Returns `None` when the
    /// point is outside the drawn viewport rectangle.
    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> Option<(f32, f32)> {
        if screen_x < self.vp_x
            || screen_x >= self.vp_x + self.vp_w
            || screen_y < self.vp_y
            || screen_y >= self.vp_y + self.vp_h
        {
            return None;
        }
        // Normalised position within the viewport in [0, 1].
        let nx = (screen_x - self.vp_x) / self.vp_w;
        // Flip Y: screen origin is top-left; world origin is bottom-left.
        let ny = 1.0 - (screen_y - self.vp_y) / self.vp_h;

        // NDC [-1, 1] → world. Uniform scale: same `half` on both axes.
        let wx = self.world_cx + (2.0 * nx - 1.0) * self.world_half;
        let wy = self.world_cy + (2.0 * ny - 1.0) * self.world_half;
        Some((wx, wy))
    }
}

// ── Input Handling ───────────────────────────────────────────────────────────

impl InputState {
    pub fn on_cursor_moved(
        &mut self,
        screen_x: f64,
        screen_y: f64,
        renderer_width: u32,
        renderer_height: u32,
        grid_size: i32,
        viewport: Option<(f32, f32, f32)>,
    ) {
        let map = ScreenMap::resolve(renderer_width, renderer_height, grid_size, viewport);
        self.mouse_grid = map
            .screen_to_world(screen_x as f32, screen_y as f32)
            .map(|(wx, wy)| {
                if is_hex_experiment(viewport) {
                    // Hex experiment: report the axial cell under the cursor so
                    // the shared hover/selection highlight lines up. The raw
                    // world coordinate is not meaningful as a square cell.
                    let (ax, ay) = hex::world_to_axial(wx, wy);
                    (ax as f32, ay as f32)
                } else {
                    (wx, wy)
                }
            });
    }

    pub fn on_mouse_button(
        &mut self,
        state: ElementState,
        button: MouseButton,
        viewport: Option<(f32, f32, f32)>,
    ) {
        if button != MouseButton::Left {
            return;
        }
        let pressed = state == ElementState::Pressed;
        // Trigger only on the leading edge of the press.
        if let (true, Some((gx, gy))) = (pressed && !self.last_mouse_down, self.mouse_grid) {
            self.selected_cell = Some(if is_hex_experiment(viewport) {
                // Axial coordinates were already stored as the integer cell.
                (gx as i32, gy as i32)
            } else {
                (gx.floor() as i32, gy.floor() as i32)
            });
        }
        self.last_mouse_down = pressed;
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────
//
// Only pure screen↔world logic is tested here; the square-grid path is
// unchanged and has no separate unit tests in this crate.

#[cfg(test)]
mod tests {
    use super::*;

    /// Default mapping (no viewport override) inverts the original square-grid
    /// projection exactly: centre pixel → world (0,0); interior edges → ±half.
    #[test]
    fn default_map_round_trips_square_grid() {
        let map = ScreenMap::resolve(1200, 800, 50, None);
        // grid area width = 1200 - 350 = 850; grid_size = 50 → half = 25.
        let (wx, wy) = map.screen_to_world(425.0, 400.0).unwrap();
        assert!((wx).abs() < 1e-4 && (wy).abs() < 1e-4, "centre -> {wx},{wy}");

        // Just inside the left edge (world x ≈ -25).
        let (wx, _) = map.screen_to_world(0.0, 400.0).unwrap();
        assert!((wx + 25.0).abs() < 1e-3, "left edge x = {wx}");
        // Just above the bottom of the screen (world y ≈ -25).
        let (_, wy) = map.screen_to_world(425.0, 799.0).unwrap();
        assert!((wy + 25.0).abs() < 0.1, "bottom edge y = {wy}");
    }

    /// An experiment viewport maps the world square onto a *square* screen
    /// rectangle centred horizontally in the grid area.
    #[test]
    fn experiment_viewport_is_square_and_centred() {
        // grid area = 850 wide, 800 tall → square side = 800, x offset = 25.
        let map = ScreenMap::resolve(1200, 800, 50, Some((0.0, 0.0, 10.0)));
        // Centre of the square viewport (x = 25 + 400 = 425, y = 400).
        let (wx, wy) = map.screen_to_world(425.0, 400.0).unwrap();
        assert!((wx).abs() < 1e-4 && (wy).abs() < 1e-4);

        // Left boundary of the square viewport (x = 25) → world -10.
        let (wx, _) = map.screen_to_world(25.0, 400.0).unwrap();
        assert!((wx + 10.0).abs() < 1e-3, "left of square -> {wx}");
        // Right boundary minus a pixel (x = 825) → world +10.
        let (wx, _) = map.screen_to_world(824.0, 400.0).unwrap();
        assert!((wx - 10.0).abs() < 0.05, "right of square -> {wx}");
        // A point in the left letterbox strip is outside the viewport.
        assert!(map.screen_to_world(10.0, 400.0).is_none());
    }

    /// The uniform (aspect-preserving) property: equal world *magnitudes* in x
    /// and y become equal screen pixel deltas (the y sign flips).
    #[test]
    fn experiment_viewport_is_isotropic() {
        let map = ScreenMap::resolve(1200, 800, 50, Some((0.0, 0.0, 10.0)));
        let (x0, y0) = map.screen_to_world(325.0, 300.0).unwrap();
        let (x1, _) = map.screen_to_world(425.0, 300.0).unwrap();
        let (_, y1) = map.screen_to_world(325.0, 400.0).unwrap();
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        assert!(
            (dx - dy).abs() < 1e-3,
            "anisotropic: |dx|={dx} |dy|={dy}"
        );
    }
}
