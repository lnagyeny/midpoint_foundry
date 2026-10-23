# Architecture

Status legend used throughout this document:

- **[verified]** — confirmed by reading the source in this repository.
- **[assumed]** — inference that has not been proven by execution.
- **[unresolved]** — open question; do not rely on a guess.

Last inspected at commit `34438a8` (branch `dev`).

---

## 1. Repository layout

```
Cargo.toml            crate manifest (binary only)
build.rs              Windows icon resource compilation (winres)
README.md             short project description
AGENTS.md             AI/project instructions
assets/icon.ico       Windows window icon
assets/icon.png       window icon, loaded at runtime for all platforms
src/main.rs           binary entry point
src/app.rs            window + app state + frame loop + algorithm registry
src/geometry.rs       algorithm output -> line/quad vertices
src/renderer.rs       WGPU pipelines, buffers, orthographic transform
src/shaders/shader.wgsl
src/input.rs          mouse -> grid mapping, cell selection
src/ui.rs             egui side panel
src/benchmark.rs      in-process algorithm timing
src/algorithms/mod.rs `Point` + `Algorithm` trait
src/algorithms/circle/
src/algorithms/ellipse/
src/algorithms/line/
src/algorithms/parabola/
examples/test_benchmark.rs   DOES NOT COMPILE (see §7)
test_benchmark_simple.rs     root-level scratch file, main() only, unused
test_refactor.rs             root-level scratch file, references a nonexistent library
.github/dependabot.yml       devcontainer ecosystem updates only
.devcontainer/devcontainer.json
```

**[verified]** There is exactly one file that declares `mod`s and a `main`:
`src/main.rs`. Cargo therefore builds a single **binary** target named
`midpoint_foundry`. There is no `src/lib.rs`.

Consequence: `use midpoint_foundry::…` never resolves from within this
project's own sources, examples, or root scripts. Internal code uses `crate::`.

---

## 2. Cargo configuration

**[verified]** `Cargo.toml`:

- `edition = "2024"` — requires a recent toolchain. Verified with
  `rustc 1.96.0-nightly` / `cargo 1.96.0-nightly`.
- Runtime dependencies: `wgpu = "22"`, `winit = "0.30"`, `egui`/`egui-wgpu`/
  `egui-winit = "0.29"`, `bytemuck = "1"` (derive), `pollster = "0.3"`,
  `env_logger = "0.11"`, `log = "0.4"`, `rayon = "1"`, `image = "0.25"`.
- Build dependency: `winres = "0.1"`, used only under
  `#[cfg(target_os = "windows")]` in `build.rs`.
- No `[lib]`, no `[[bin]]`, no `[features]`, no `[profile]`, no `[workspace]`.

`rayon` is a dependency used by at least one parallel algorithm
(`parallel_midpoint_circle.rs`). It is **not** used to render anything.

---

## 3. Entry point and event flow

**[verified]** `src/main.rs`:

1. `env_logger::init()`.
2. Creates a `winit` `EventLoop`, sets `ControlFlow::Poll`, runs
   `app::Application` (which is `#[derive(Default)]` with `state: Option<AppState>`).

**[verified]** `src/app.rs`:

- `Application::resumed` creates the window once (title "Midpoint Foundry",
  1200×800, resizable, icon from `assets/icon.png` via the `image` crate).
- `AppState::new_blocking` uses `pollster::block_on(Renderer::new(...))`,
  builds an `egui::Context`, an `egui_winit::State`, an `egui_wgpu::Renderer`,
  and the algorithm registry, then computes every algorithm once.
- Per frame (`WindowEvent::RedrawRequested`):
  1. `take_egui_input` → `ctx.run(...)` → `ui::draw_panel(...)`, which returns
     whether a parameter changed and a recompute is needed.
  2. If needed, `algorithms[selected].compute()`.
  3. `geometry::build(...)` produces `line_verts` and `quad_verts`.
  4. `Renderer::render(...)` draws the grid pass, then the egui pass.
  5. `request_redraw()` is called after every event (continuous animation loop).
- Keyboard: `Escape` exits. `CursorMoved` and `MouseInput` are forwarded to
  `InputState` **only when egui did not consume the event**.
- Window constants: `WINDOW_W = 1200`, `WINDOW_H = 800`, `UI_W = 350.0`
  (right panel width in physical pixels).

**[verified]** The algorithm registry is a `Vec<Box<dyn Algorithm>>` built in
`AppState::new_blocking`, in this order:

| # | Struct | Category | Default state |
|---|--------|----------|---------------|
| 0 | `BaselineCircle` | circle | centre (0,0), r = 15 |
| 1 | `MidpointCircle` | circle | r = 15 |
| 2 | `GapFillCircle` | circle | r = 15 |
| 3 | `MidpointLine` | line | — |
| 4 | `MidpointParabola` | parabola | k = 0.1 |
| 5 | `MidpointEllipse` | ellipse | 30 × 20 |
| 6 | `ParallelMidpointCircle` | circle | r = 15 |
| 7 | `SitaramanFillCircle` | circle | r = 15 |
| 8 | `EnhancedFillCircle` | circle | r = 15 |
| 9 | `RogetHCircle` | circle | r = 15, h = 1.0 |
| 10 | `ScanlineCircle` | circle | r = 15 |
| 11 | `GapFillEllipse` | ellipse | 30 × 20 |
| 12 | `VirtualFillEllipse` | ellipse | 30 × 20 |
| 13 | `RolandFillEllipse` | ellipse | 30 × 20 |
| 14 | `OwnFillEllipse` | ellipse | 30 × 20 |
| 15 | `OwnDecisionEllipse` | ellipse | 30 × 20 |

Adding a new algorithm currently means: create the file, add a `pub mod` line
to the right `mod.rs`, and push a `Box::new(...)` into this vector.

---

## 4. The `Algorithm` trait

**[verified]** `src/algorithms/mod.rs` defines:

```rust
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
pub struct Point { pub x: i32, pub y: i32 }

pub trait Algorithm: Send + Sync {
    fn name(&self) -> &str;
    fn compute(&mut self);
    fn points(&self) -> &[Point];
    fn color(&self) -> [f32; 3] { [1.0, 0.0, 0.0] }
    fn point_color_override(&self, index: usize) -> Option<[f32; 3]> { None }
    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool;   // true => recompute
    fn overlay_circle(&self) -> Option<(i32, i32, i32)> { None }
    fn overlay_parabola(&self) -> Option<(i32, i32, i32)> { None }
    fn overlay_ellipse(&self) -> Option<(i32, i32, i32, i32)> { None }
    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> { None }
    fn category(&self) -> &'static str;
}
```

Key design facts:

- Coordinate space is a **square integer lattice**. One `Point` = one cell.
- `Send + Sync` are required because `compute()` can run across threads
  (`rayon`) and the trait objects are stored in the app.
- Overlays are integer descriptions of the ideal analytic shape; `geometry.rs`
  tessellates them into white line segments for visual comparison.
- `draw_ui` returning `true` is the only signalling mechanism for "parameters
  changed; recompute now". There is no dirty-flag cache beyond this, except the
  duplicate-detection cache in the base structs.
- `cell_info` is used by the UI to print the implicit-equation value of the
  hovered/selected cell (see `CircleBase::cell_info_base`).

**Geometry/algorithm coupling [verified]:** the y axis in grid space points up,
and `input.rs` flips the screen y. A cell at integer `(x, y)` is drawn as the
unit square from `(x, y)` to `(x+1, y+1)`. The analytic overlay centres are
drawn at `(cx + 0.5, cy + 0.5)`. So the integer coordinates name **cell
indices**, and the analytic curve passes through the centre of the named cell.

---

## 5. Rendering pipeline

**[verified]** `src/geometry.rs` → `Renderer::render`:

Vertex type (`renderer.rs`), `#[repr(C)]`, `bytemuck::Pod`:

```
pos:   [f32; 2]     // grid-space position
color: [f32; 4]     // RGBA, alpha used for translucent highlights
```

Two vertex streams:

- `quad_verts` → `TriangleList` pipeline. Contains: analytic fill background
  (white disc/ellipse), algorithm points (coloured per point), hover
  row/column highlight, selected-cell highlight. Drawn **first**.
- `line_verts` → `LineList` pipeline. Contains: grid lines, axes, the 45°
  diagonal, and the analytic circle/parabola/ellipse overlays. Drawn **after**
  quads, so lines appear on top.

Pipeline setup:

- WGSL `src/shaders/shader.wgsl` is trivial: `vs_main` multiplies a 2D
  position by a diagonal `mat4x4<f32>` uniform, `fs_main` returns the vertex
  colour unchanged.
- Uniform: `Uniforms::ortho_centered(cx, cy, half)` maps grid coordinates
  `[cx−half, cx+half] × [cy−half, cy+half]` to NDC using the **same** scale
  `1/half` on both axes. For the square grid the caller passes
  `(cx, cy, half) = (0, 0, grid_size/2)`, which is exactly the old
  `Uniforms::ortho(half)`.
- **[verified]** A defaulted `Algorithm::viewport()` (default `None`) lets a
  research experiment request an *aspect-preserving* view. When `Some((cx,cy,h))`,
  `Renderer::render` maps the world square to a **square** physical viewport of
  side `min(area_width, height)` centred horizontally, so hexagons stay regular
  and circles stay circular (letterboxed otherwise). The values are carried in a
  small `renderer::GridView` descriptor. Default `None` keeps the full-area
  viewport `(0, 0, width − 350, height)` unchanged.
- Blending is `ALPHA_BLENDING`; cull mode is `None`; no depth buffer.
- The grid pass sets the viewport resolved above, so the egui side panel does
  not cover the grid. Background is cleared to dark grey
  `(0.1, 0.1, 0.1, 1.0)`.
- Vertex buffers start at 32 768 vertices and grow to the next power of two
  on demand (`ensure_line_capacity` / `ensure_quad_capacity`).
- After the grid pass, `egui` is tessellated and drawn with `LoadOp::Load`,
  preserving the grid underneath.

**[verified]** `grid_size` is a `Slider` in `10..=200` with `step_by(2.0)`,
default 50. It is used as a square: `half = grid_size / 2`, with grid lines
from `-half` to `+half` on both axes.

**[verified]** The whole grid is redrawn every frame; there is no vertex
caching keyed on algorithm state.

---

## 6. Input

**[verified]** `src/input.rs`:

- `on_cursor_moved(screen_x, screen_y, renderer_width, renderer_height, grid_size, viewport)`.
  A pure `ScreenMap::resolve(...)` mirrors the renderer transform: it records
  the world rectangle and the screen rectangle drawn into it. The default
  (no `viewport` override) reproduces the original square-grid mapping
  (`gx = screen_x / grid_w * grid_size − half`, y flipped). An override maps the
  world square to the same square screen viewport the renderer uses.
- When the override is present (the hex experiment), the cursor's world
  coordinate is converted to an axial hex cell with `hex::world_to_axial` (cube
  rounding) before being stored, so hover/selection line up with the drawn cells.
- `on_mouse_button` selects the cell on the leading edge of a left press:
  `selected_cell = (floor(gx), floor(gy))` (axial cell for the experiment).
- **[verified]** The default square-grid path still does **not** correct the
  aspect ratio (unchanged); only an experiment `viewport()` override does.

---

## 7. Test and benchmark infrastructure

**[verified]** Current state:

- `cargo test --bins` runs **22 tests** (17 in `src/algorithms/hex/mod.rs`,
  5 in `src/input.rs`) and reports `ok`. `cargo test` / `--all-targets` /
  `check --all-targets` remain broken because `examples/test_benchmark.rs`
  imports a non-existent library target.
- `src/benchmark.rs` is *not* a test harness. It is an in-process timer:
  `run_benchmark(radius)` instantiates a subset of algorithms, warms each up,
  runs 100 iterations of `compute()`, and returns `min/max/avg/median`
  microseconds. It is triggered from the egui panel ("📊 Benchmark").
  It only covers circle algorithms plus `MidpointLine`. It does **not** cover
  ellipse or parabola algorithms.
- `examples/test_benchmark.rs` calls `midpoint_foundry::benchmark::run_benchmark`.
  **[verified]** This fails to compile:
  `error[E0433]: cannot find module or crate midpoint_foundry`. As a result,
  `cargo test`, `cargo test --all-targets`, and `cargo check --all-targets`
  fail. Only `cargo check`, `cargo build`, and `cargo test --bins` succeed.
- `test_benchmark_simple.rs` and `test_refactor.rs` are root-level files that
  Cargo does not build (not under `src/`, `tests/`, `examples/`, or `benches/`).
  `test_refactor.rs` also contains `use midpoint_foundry::…` and would not
  compile if it were ever included. **[verified]** They are dead scratch files.
- There is **no** `tests/` integration-test directory and **no** `benches/`
  directory, despite `Cargo.lock` listing `criterion` as a package
  (transitive/unused; `Cargo.toml` has no dev-dependencies). [verified]

**Implication for the research module:** a real, `cargo test`-runnable test
target does not exist yet. New tests should be added as `#[cfg(test)] mod tests`
inside `src/` (run with `cargo test --bins`) unless and until a library target
is introduced. Introducing a `src/lib.rs` would also fix `examples/`, but that
is a structural decision, not a casual change — see `docs/ai/DECISIONS.md`.

---

## 8. Existing colour / UI conventions

**[verified]**

- `CircleBase::PALETTE` (7 colours) and `EllipseBase::PALETTE_7` (identical
  values) are ring-colour palettes for fill-style algorithms.
- Duplicate detection is implemented twice: `CircleBase::duplicate_color` and
  `EllipseBase::duplicate_color`. Both lazily build a `HashSet<(i32,i32)>` of
  coordinates that occur more than once and colour those points red
  `[1.0, 0.0, 0.0]` when "Highlight duplicates" is enabled.
- `EllipseBase::invalidate_cache` must be called when `points` is rebuilt;
  `draw_ui_full` does this when a control changes.
  `OwnDecisionEllipse` clears `point_rings` and `points` in `compute()`.
- Ring colouring uses `point_rings[r % N]`, i.e. it wraps. The ellipse
  algorithms inline `PALETTE_7[r % 7]`, so they do not use a shared helper for
  this.

**[verified]** `EllipseBase::ring_color` and
`EllipseBase::draw_ui_with_duplicates` were unused dead code and have been
**removed** (DR-010); the resulting `clippy` dead-code warning is gone. No other
`EllipseBase` helper changed.

---

## 9. Square-grid assumptions to be aware of

These are the places where the current architecture hard-codes a square grid.
They matter for the hexagonal research module.

1. **Cell shape.** `geometry::push_quad` always emits a unit square. A hex cell
   (or any other cell shape) has no representation. **[verified]**
2. **Grid rendering.** §4/§5: grid lines are drawn at every integer `x` and `y`
   across `[-half, +half]`, giving a square lattice. **[verified]**
3. **Coordinate mapping.** The default `input.rs` path maps the mouse to
   `(floor(gx), floor(gy))` independently per axis — valid for squares, wrong for
   hex offsets. **[verified]** An experiment `viewport()` override routes the
   cursor through `hex::world_to_axial` instead (DR-009); the square-grid path is
   unchanged.
4. **Aspect ratio.** The default projection is per-axis `1/half`; the grid is
   stretched to the viewport, so cells are not square on screen. **[verified]**
   The renderer now supports an aspect-preserving square viewport via an
   experiment `viewport()` override (DR-009); the default is unchanged.
5. **`Point { x: i32, y: i32 }`** carries no coordinate-system tag. A hex-cell
   coordinate (axial/cube) cannot be distinguished from a square pixel by type.
   **[verified]**
6. **Overlays** are specialised to circles, parabolas, and axis-aligned
   ellipses. There is no generic curve overlay. **[verified]**
7. **`grid_size`** is a scalar meaning "side length of the square grid".
   **[verified]**

The square-grid algorithms themselves (`circle/*`, `ellipse/*`, `line/*`,
`parabola/*`) are self-contained; they only read their own fields and push
`Point`s. They do not assume anything about rendering beyond integer cell
coordinates. **[verified]**

---

## 10. Reusable components for hexagonal experiments

**[verified] reusable as-is:**

- `algorithms::Point`, `algorithms::Algorithm` trait, and the registry pattern
  in `app.rs` — a hex algorithm can be an `Algorithm` implementation.
- `ui::draw_panel` category/algorithm filtering and the benchmark panel.
- `benchmark::run_benchmark` pattern (timing an in-process `compute()`), though
  it currently hard-codes the algorithm list.
- `renderer.rs` line/quad pipelines and `Vertex`.
- The duplicate-detection idea in the base structs (as a *technique*, not
  necessarily the code).

**[verified] reusable with modification:**

- `geometry::build` — the point loop (§5 `"Algorithm points"`) works unchanged
  for any integer-cell algorithm. The grid-line drawing, hover highlight,
  square-cell quads, and overlays are square-specific and would need a hex
  variant or a separate builder.
- `input.rs` — needs a hex-aware inverse mapping (pixel → hex cell).

**[assumed, needs work] a hex renderer:** there is no existing helper to emit a
hexagon polygon, an oblique/hex projection, or a hex grid outline.

---

## 11. Open questions / unresolved

- **[unresolved]** Should the hex module be a new `Algorithm` category
  (`"hex"`) inside the existing app, or a separate executable/example? No
  decision has been made.
- **[unresolved]** Whether to introduce `src/lib.rs` so that integration tests
  and `examples/` can depend on the crate. This would also fix the currently
  broken `examples/test_benchmark.rs`.
- **[unresolved]** Exact screen projection for regular hexagons (aspect-ratio
  correction vs. an oblique grid like the one in the research spec).
- **[unresolved]** Whether hex cells will reuse the unit-square `Vertex` quad
  path with a per-cell polygon emitter, or a new pipeline.
- **[unresolved]** Whether `grid_size: i32` is reinterpreted for hex or a
  separate radius/size control is added.
