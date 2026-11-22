# Project state

**Read this first, every session.** Short snapshot only. History lives in
`SESSION_LOG.md`; rationale lives in `DECISIONS.md`; details live in
`docs/architecture.md` and `docs/research/hexagonal-grid.md`.

**Last updated:** 2026-11-23
**Branch:** `dev`  **HEAD:** `e8009d5` ("added ellipse, hexa view, agents, skills")
**Working tree:** uncommitted hex research changes (pointy-top orientation,
corrected midpoint circle, new layered gap-fill circle). No commit/push made.
Untracked `.devcontainer/`, `.github/`, `.gitignore` are user files — do not touch.

---

## Current research objective

Produce and validate midpoint-style rasterisation for **hexagonal cell grids**.
The selectable `MidpointHexCircle` implements the supplied paper's 30° sector
and 12-way integer symmetry; `LayeredGapFillCircle` is an independent
concentric-band experiment. Research spec §11 still has an open boundary /
stop-condition question (§11.6). §12 and DR-011 document the hex experiment,
rendering and pointy-top orientation.

## Implemented and verified functionality

- Working Rust/WGPU/winit/egui desktop app (`cargo check`, `cargo build` pass;
  previously launched on `llvmpipe` without panicking, window not inspected).
- 16 square-grid algorithms plus three selectable hex entries: the visualisation
  experiment, `MidpointHexCircle` and `LayeredGapFillCircle`.
- Square-grid CPU rasterisation → vertex geometry → WGPU draw, egui panel,
  hover/selection, analytic overlays, duplicate highlighting, benchmark button.
- **51 unit tests**, all passing via `cargo test --bins` and
  `cargo test --all-targets`: 22 in `src/algorithms/hex/mod.rs`, 14 in
  `midpoint_hex_circle.rs`, 9 in `layered_gap_fill_circle.rs`, 6 in `input.rs`.
- Hex experiments bypass the square path with `overlay_geometry` and a square,
  aspect-preserving `viewport()`; mouse input converts to axial cells via
  `world_to_hex`. Pointy-top is render/input-only (DR-011); mapping A, `F`,
  neighbours and all square-grid algorithms are unchanged.

## Architecture relevant to the research

- Both a library target (`src/lib.rs`) and the binary exist; crate imports may
  use `crate::…` or `midpoint_foundry::…`. `examples/test_benchmark.rs` compiles.
- `Algorithm` trait + `Vec<Box<dyn Algorithm>>` registry in `src/app.rs`.
- `Point { x: i32, y: i32 }` is a grid cell; square-grid cells are unit quads.
- Hex geometry is built in each experiment via `push_hex_fill` /
  `push_hex_outline`; `ExperimentGeometry` carries line and quad vertex buffers.

## Decisions already made

- OpenCode-native project skills; reviewer is read-only.
- DR-008: `overlay_geometry` isolates hex rendering from the square path.
- DR-011: pointy-top as a +30° render/input rotation only.
- DR-012: layered gap-fill uses midpoint-ring unions and per-layer colours.
- Full list: `docs/ai/DECISIONS.md`.

## Known limitations and unresolved issues

- Repository is not `cargo fmt`-clean (pre-existing drift in `app.rs`,
  `algorithms/hex/mod.rs`, `input.rs`, `benchmark.rs`, `ui.rs`); the new
  `layered_gap_fill_circle.rs` is rustfmt-clean. `cargo clippy --bins` has 6
  pre-existing warnings, none in newly added code.
- No screenshot tool available, so hex appearance is verified numerically only;
  user-side visual confirmation is still outstanding.
- **Open mathematical question:** spec §11.6 boundary definition (single closest
  cell vs bracketing pair) and the derived sector stop condition. The
  selectable midpoint algorithm is implemented but is not claimed to settle it.
- No general algorithm test harness; tests live in `#[cfg(test)]` modules in `src/`.

## Next concrete task

User-side visual confirmation of pointy-top rendering and the layered bands,
then research §11.6 using the hex experiment as the reference.

## Exact verification commands to run

```sh
git status                                   # confirm no unexpected changes
cargo check                                  # typecheck (must pass)
cargo test --bins                            # 51 tests; must pass
cargo test --all-targets                     # 51 tests; must pass
cargo clippy --bins                          # 6 pre-existing warnings only
cargo fmt --check                            # expected FAIL (pre-existing drift)
```

## Do not touch (pre-existing user changes)

- Untracked: `.devcontainer/`, `.github/`, `.gitignore`.
- Read-only references: `src/algorithms/circle/gap_fill_circle.rs`,
  `midpoint_hex_circle.rs`, all ellipse/square-grid algorithms, renderer,
  global colours.
