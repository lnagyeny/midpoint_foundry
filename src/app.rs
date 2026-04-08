use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{
    algorithms::{
        baseline_circle::BaselineCircle, gap_fill_circle::GapFillCircle,
        gap_fill_ellipse::GapFillEllipse, midpoint_circle::MidpointCircle,
        midpoint_ellipse::MidpointEllipse, midpoint_line::MidpointLine,
        midpoint_parabola::MidpointParabola, parallel_midpoint_circle::ParallelMidpointCircle,
        scanline_circle::ScanlineCircle, sitaraman_fill_circle::SitaramanFillCircle,
        virtual_fill_ellipse::VirtualFillEllipse, Algorithm,
    },
    geometry,
    input::InputState,
    renderer::Renderer,
    ui::{draw_panel, UiState},
};

// ── Window / viewport constants ───────────────────────────────────────────────

const WINDOW_W: u32 = 1200;
const WINDOW_H: u32 = 800;
const UI_W: f32 = 350.0; // right-panel pixel width

// ── AppState ──────────────────────────────────────────────────────────────────

struct AppState {
    window: Arc<Window>,
    renderer: Renderer,
    egui_ctx: egui::Context,
    egui_winit: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,

    algorithms: Vec<Box<dyn Algorithm>>,
    selected: usize,
    grid_size: i32,

    ui_state: UiState,
    input_state: InputState,
}

impl AppState {
    fn new_blocking(window: Arc<Window>) -> Self {
        let renderer = pollster::block_on(Renderer::new(Arc::clone(&window)));

        let egui_ctx = egui::Context::default();
        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            None, // native_pixels_per_point
            None, // max_texture_side
            None, // max_texture_side in this version (new optional parameter)
        );
        let egui_renderer =
            egui_wgpu::Renderer::new(&renderer.device, renderer.surface_format, None, 1, true);

        // Instantiate and compute all algorithms up-front.
        let algorithms: Vec<Box<dyn Algorithm>> = vec![
            Box::new(BaselineCircle::default()),
            Box::new(MidpointCircle::default()),
            Box::new(GapFillCircle::default()),
            Box::new(MidpointLine::default()),
            Box::new(MidpointParabola::default()),
            Box::new(MidpointEllipse::default()),
            Box::new(ParallelMidpointCircle::default()),
            Box::new(SitaramanFillCircle::default()),
            Box::new(ScanlineCircle::default()),
            Box::new(GapFillEllipse::default()),
            Box::new(VirtualFillEllipse::default()),
        ];

        Self {
            window,
            renderer,
            egui_ctx,
            egui_winit,
            egui_renderer,
            algorithms,
            selected: 0,
            grid_size: 50,
            ui_state: UiState::default(),
            input_state: InputState::default(),
        }
    }

    // ── Frame ─────────────────────────────────────────────────────────────────

    fn render(&mut self) {
        // `egui::Context` is an Arc wrapper — clone is O(1) and shares state.
        // We must clone here so that `run()` doesn't hold a borrow on `self`
        // while the closure also needs `&mut self` to call draw_panel.
        let raw_input = self.egui_winit.take_egui_input(&self.window);
        let ctx = self.egui_ctx.clone();
        let mut needs_recompute = false;
        let egui_output = ctx.run(raw_input, |ctx| {
            needs_recompute = draw_panel(
                ctx,
                &mut self.ui_state,
                &mut self.algorithms,
                &mut self.selected,
                &mut self.grid_size,
                self.input_state.selected_cell,
            );
        });
        self.egui_winit
            .handle_platform_output(&self.window, egui_output.platform_output.clone());

        if needs_recompute {
            self.algorithms[self.selected].compute();
        }

        // Build vertex geometry.
        let geo = geometry::build(
            &*self.algorithms[self.selected],
            self.grid_size,
            self.input_state.mouse_grid,
            self.input_state.selected_cell,
        );

        let grid_area_w = self.renderer.width as f32 - UI_W;
        let grid_half = self.grid_size as f32 / 2.0;

        self.renderer.render(
            &geo.line_verts,
            &geo.quad_verts,
            grid_area_w,
            grid_half,
            egui_output,
            &mut self.egui_renderer,
            &self.egui_ctx,
        );
    }
}

// ── ApplicationHandler ────────────────────────────────────────────────────────

#[derive(Default)]
pub struct Application {
    state: Option<AppState>,
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("Midpoint Foundry")
            .with_inner_size(PhysicalSize::new(WINDOW_W, WINDOW_H))
            .with_resizable(true);

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create window"),
        );
        self.state = Some(AppState::new_blocking(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        // Forward to egui; check if it consumed the event.
        let egui_consumed = state
            .egui_winit
            .on_window_event(&state.window, &event)
            .consumed;

        match &event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                use winit::keyboard::{KeyCode, PhysicalKey};
                if event.physical_key == PhysicalKey::Code(KeyCode::Escape) {
                    event_loop.exit();
                }
            }

            WindowEvent::Resized(size) => {
                state.renderer.resize(size.width, size.height);
            }

            WindowEvent::CursorMoved { position, .. } if !egui_consumed => {
                state.input_state.on_cursor_moved(
                    position.x,
                    position.y,
                    state.renderer.width,
                    state.renderer.height,
                    state.grid_size,
                );
            }

            WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } if !egui_consumed => {
                state.input_state.on_mouse_button(*btn_state, *button);
            }

            WindowEvent::RedrawRequested => {
                state.render();
            }

            _ => {}
        }

        state.window.request_redraw();
    }
}
