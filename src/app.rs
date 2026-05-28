use std::sync::Arc;
#[cfg(target_os = "windows")]
use winit::platform::windows::WindowAttributesExtWindows;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Icon, Window, WindowId},
};

use crate::{
    algorithms::{
        circle::baseline_circle::BaselineCircle, circle::enhanced_fill_circle::EnhancedFillCircle,
        circle::gap_fill_circle::GapFillCircle, circle::midpoint_circle::MidpointCircle,
        circle::parallel_midpoint_circle::ParallelMidpointCircle,
        circle::scanline_circle::ScanlineCircle,
        circle::sitaraman_fill_circle::SitaramanFillCircle,
        ellipse::gap_fill_ellipse::GapFillEllipse, ellipse::midpoint_ellipse::MidpointEllipse,
        ellipse::own_decision_ellipse::OwnDecisionEllipse,
        ellipse::own_fill_ellipse::OwnFillEllipse, ellipse::roland_fill_ellipse::RolandFillEllipse,
        ellipse::virtual_fill_ellipse::VirtualFillEllipse, line::midpoint_line::MidpointLine,
        parabola::midpoint_parabola::MidpointParabola, Algorithm,
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
            Box::new(EnhancedFillCircle::default()),
            Box::new(ScanlineCircle::default()),
            Box::new(GapFillEllipse::default()),
            Box::new(VirtualFillEllipse::default()),
            Box::new(RolandFillEllipse::default()),
            Box::new(OwnFillEllipse::default()),
            Box::new(OwnDecisionEllipse::default()),
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

        fn load_icon() -> Option<Icon> {
            let bytes = include_bytes!("../assets/icon.png");
            let img = image::load_from_memory(bytes).ok()?.into_rgba8();
            let (w, h) = img.dimensions();
            Icon::from_rgba(img.into_raw(), w, h).ok()
        }

        let attrs = Window::default_attributes()
            .with_title("Midpoint Foundry")
            .with_window_icon(load_icon())
            .with_inner_size(PhysicalSize::new(WINDOW_W, WINDOW_H))
            .with_resizable(true);

        #[cfg(target_os = "windows")]
        let attrs = attrs.with_taskbar_icon(load_icon());

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
