use winit::event::{ElementState, MouseButton};

// ── Input State ──────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct InputState {
    pub mouse_grid: Option<(f32, f32)>,
    pub selected_cell: Option<(i32, i32)>,
    pub last_mouse_down: bool,
}

// ── Input Handling ───────────────────────────────────────────────────────────

impl InputState {
    pub fn on_cursor_moved(&mut self, screen_x: f64, screen_y: f64, renderer_width: u32, renderer_height: u32, grid_size: i32) {
        const UI_W: f32 = 350.0; // right-panel pixel width
        let grid_w = renderer_width as f32 - UI_W;

        if screen_x >= 0.0
            && screen_x < grid_w as f64
            && screen_y >= 0.0
            && screen_y < renderer_height as f64
        {
            let half = grid_size as f32 / 2.0;
            let gx = (screen_x as f32 / grid_w) * grid_size as f32 - half;
            // Flip Y: screen origin is top-left; grid origin is bottom-left.
            let gy = ((renderer_height as f64 - screen_y) as f32
                / renderer_height as f32)
                * grid_size as f32
                - half;
            self.mouse_grid = Some((gx, gy));
        } else {
            self.mouse_grid = None;
        }
    }

    pub fn on_mouse_button(&mut self, state: ElementState, button: MouseButton) {
        if button != MouseButton::Left {
            return;
        }
        let pressed = state == ElementState::Pressed;
        // Trigger only on the leading edge of the press.
        if pressed && !self.last_mouse_down {
            if let Some((gx, gy)) = self.mouse_grid {
                self.selected_cell = Some((gx.floor() as i32, gy.floor() as i32));
            }
        }
        self.last_mouse_down = pressed;
    }
}