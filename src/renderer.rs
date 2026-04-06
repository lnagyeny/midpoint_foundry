use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use winit::window::Window;

// ── Vertex type ───────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4], // RGBA — alpha < 1 for transparent highlights
}

impl Vertex {
    pub fn new(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            pos: [x, y],
            color: [r, g, b, a],
        }
    }

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

// ── Uniform ───────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    /// Orthographic projection: grid coords [-half, +half] → NDC [-1, +1].
    fn ortho(half: f32) -> Self {
        let s = 1.0 / half;
        Self {
            view_proj: [
                [s, 0.0, 0.0, 0.0],
                [0.0, s, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

// ── Renderer ──────────────────────────────────────────────────────────────────

/// Maximum vertices per buffer (each is resized upward if exceeded).
const INITIAL_CAPACITY: usize = 32_768;

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,

    line_pipeline: wgpu::RenderPipeline,
    quad_pipeline: wgpu::RenderPipeline,

    line_buf: wgpu::Buffer,
    line_cap: usize,
    quad_buf: wgpu::Buffer,
    quad_cap: usize,

    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,

    pub width: u32,
    pub height: u32,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("Failed to create wgpu surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No suitable GPU adapter found");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None, // optional trace path in this wgpu version
            )
            .await
            .expect("Failed to create wgpu device");

        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // ── Shader ────────────────────────────────────────────────────────────
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        // ── Uniform buffer & bind group ───────────────────────────────────────
        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pl"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        // ── Helper: build a render pipeline ───────────────────────────────────
        let make_pipeline = |topology: wgpu::PrimitiveTopology| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(match topology {
                    wgpu::PrimitiveTopology::LineList => "line_pipeline",
                    _ => "quad_pipeline",
                }),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        };

        let line_pipeline = make_pipeline(wgpu::PrimitiveTopology::LineList);
        let quad_pipeline = make_pipeline(wgpu::PrimitiveTopology::TriangleList);

        // ── Vertex buffers ────────────────────────────────────────────────────
        let line_buf = Self::make_vertex_buf(&device, INITIAL_CAPACITY, "line_vb");
        let quad_buf = Self::make_vertex_buf(&device, INITIAL_CAPACITY, "quad_vb");

        Self {
            device,
            queue,
            surface,
            surface_config,
            surface_format,
            line_pipeline,
            quad_pipeline,
            line_buf,
            line_cap: INITIAL_CAPACITY,
            quad_buf,
            quad_cap: INITIAL_CAPACITY,
            uniform_buf,
            bind_group,
            width: size.width,
            height: size.height,
        }
    }

    // ── Public API ─────────────────────────────────────────────────────────────

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.width = width;
        self.height = height;
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    /// Upload geometry, run the grid render pass, then the egui render pass.
    pub fn render(
        &mut self,
        line_verts: &[Vertex],
        quad_verts: &[Vertex],
        grid_area_width: f32, // physical pixels of the grid viewport
        grid_half: f32,       // half the grid size in grid-space units
        egui_output: egui::FullOutput,
        egui_renderer: &mut egui_wgpu::Renderer,
        egui_ctx: &egui::Context,
    ) {
        // Ensure vertex buffers are large enough.
        self.ensure_line_capacity(line_verts.len().max(1));
        self.ensure_quad_capacity(quad_verts.len().max(1));

        // Upload uniforms.
        let uniforms = Uniforms::ortho(grid_half);
        self.queue
            .write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&[uniforms]));

        // Upload vertices.
        if !line_verts.is_empty() {
            self.queue
                .write_buffer(&self.line_buf, 0, bytemuck::cast_slice(line_verts));
        }
        if !quad_verts.is_empty() {
            self.queue
                .write_buffer(&self.quad_buf, 0, bytemuck::cast_slice(quad_verts));
        }

        let surface_tex = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(e) => {
                log::warn!("get_current_texture: {e}");
                return;
            }
        };
        let view = surface_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        // ── Grid / algorithm render pass ──────────────────────────────────────
        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("grid_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rp.set_viewport(0.0, 0.0, grid_area_width, self.height as f32, 0.0, 1.0);
            rp.set_bind_group(0, &self.bind_group, &[]);

            // Quads first (points, highlights) — behind grid lines.
            if !quad_verts.is_empty() {
                rp.set_pipeline(&self.quad_pipeline);
                rp.set_vertex_buffer(0, self.quad_buf.slice(..));
                rp.draw(0..quad_verts.len() as u32, 0..1);
            }

            // Lines on top (grid, axes, overlay circle).
            if !line_verts.is_empty() {
                rp.set_pipeline(&self.line_pipeline);
                rp.set_vertex_buffer(0, self.line_buf.slice(..));
                rp.draw(0..line_verts.len() as u32, 0..1);
            }
        }

        // ── egui render pass ──────────────────────────────────────────────────
        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.width, self.height],
            pixels_per_point: egui_ctx.pixels_per_point(),
        };

        for (id, delta) in egui_output.textures_delta.set {
            egui_renderer.update_texture(&self.device, &self.queue, id, &delta);
        }

        let primitives = egui_ctx.tessellate(egui_output.shapes, egui_ctx.pixels_per_point());
        egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &primitives,
            &screen_desc,
        );

        {
            let rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // preserve grid render
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let mut rp = rp.forget_lifetime();
            egui_renderer.render(&mut rp, &primitives, &screen_desc);
        }

        for id in egui_output.textures_delta.free {
            egui_renderer.free_texture(&id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_tex.present();
    }

    // ── Private helpers ────────────────────────────────────────────────────────

    fn make_vertex_buf(device: &wgpu::Device, capacity: usize, label: &str) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (std::mem::size_of::<Vertex>() * capacity) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn ensure_line_capacity(&mut self, needed: usize) {
        if needed > self.line_cap {
            let new_cap = needed.next_power_of_two();
            self.line_buf = Self::make_vertex_buf(&self.device, new_cap, "line_vb");
            self.line_cap = new_cap;
        }
    }

    fn ensure_quad_capacity(&mut self, needed: usize) {
        if needed > self.quad_cap {
            let new_cap = needed.next_power_of_two();
            self.quad_buf = Self::make_vertex_buf(&self.device, new_cap, "quad_vb");
            self.quad_cap = new_cap;
        }
    }
}
