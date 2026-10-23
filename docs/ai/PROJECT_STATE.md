# Project state

**Read this first, every session.** Short snapshot only. History lives in
`SESSION_LOG.md`; rationale lives in `DECISIONS.md`; details live in
`docs/architecture.md` and `docs/research/hexagonal-grid.md`.

**Last updated:** 2026-10-22
**Branch:** `dev`  **HEAD:** `34438a8` ("added Roget based enhanced H circle algorithm")
**Working tree:** uncommitted changes from the hex orientation + dead-code
cleanup (see below) plus one pre-existing modification
(`src/algorithms/ellipse/mod.rs`) and pre-existing untracked files — see
"Do not touch" below.

---

## Current research objective

Produce and validate **midpoint-style algorithms for hexagonal cell grids**.
No midpoint algorithm exists or has been derived yet.
`docs/research/hexagonal-grid.md` §11 holds the first **proposed model v0** (for
review) with a **blocking unresolved item** (§11.6). §12 documents **Experiment 1**
(hex grid visualisation + brute-force reference classification), which is
**implemented and tested**.

## Implemented and verified functionality

- Working Rust/WGPU/winit/egui desktop app (verified: `cargo check`,
  `cargo build` succeed; launched this session on `llvmpipe`, ran without
  panicking — window not visually inspected).
- 16 square-grid algorithms across `circle`, `ellipse`, `line`, `parabola`,
  plus **1 hex research experiment** (`hex`, not an algorithm).
- Square-grid CPU rasterisation → vertex geometry → WGPU draw, plus egui panel,
  hover/selection, analytic overlays, duplicate highlighting, benchmark button.
- **22 unit tests now exist** (17 in `src/algorithms/hex/mod.rs`, 5 in
  `src/input.rs`; run via `cargo test --bins`; all pass). Hex grid: axial
  mapping, neighbours, quadratic form, hexagon tiling, classification, geometry
  counts, region coverage/duplicates/adjacency, coordinate round-trip and
  nearest-cell hit test. Input: square-grid and experiment screen mapping.
- Isolated experiment rendering: `Algorithm::overlay_geometry` (default `None`)
  lets a research algorithm supply complete custom geometry. All square-grid
  algorithms are unaffected.
- Aspect-correct experiment view: defaulted `Algorithm::viewport()` (default
  `None`) + `renderer::GridView`. When `Some`, the world square is mapped to a
  **square** screen viewport with a single uniform scale (letterboxed), so cells
  stay regular and circles stay circular. Default `None` reproduces the original
  square-grid projection byte-for-byte.
- Hex-aware mouse input: when the experiment viewport is active, `input.rs`
  converts the cursor to an axial cell via `hex::world_to_axial` (cube
  rounding), so hover/selection and the panel's "Selected cell" line up. The
  square-grid input path is unchanged.

## Architecture relevant to the research

- Binary crate only; **no `lib.rs`**, so `use midpoint_foundry::…` fails.
- `Algorithm` trait + `Vec<Box<dyn Algorithm>>` registry in `src/app.rs`.
- `Point { x: i32, y: i32 }` = one square cell; cells drawn as unit quads.
- Grid lines, hover mapping and projection are square-specific
  (`docs/architecture.md` §9).
- Hex rendering bypasses the square path via `overlay_geometry`; categories are
  discovered dynamically by the UI, so `"hex"` needs no UI change.
- Known-broken build targets: `examples/test_benchmark.rs` (unresolved crate),
  `test_*.rs` scratch files. Use `cargo test --bins`.

## Decisions already made

- Use OpenCode-native project skills under `.opencode/skills/`.
- Reviewer is a **read-only** subagent; no multi-agent orchestration yet.
- Research spec §11 = proposed model v0 (**not yet approved / not verified**).
- DR-008: hex experiment isolated via a defaulted `overlay_geometry` trait
  method; no square-grid rendering change.
- Full list: `docs/ai/DECISIONS.md`.

## Known limitations and unresolved issues

- No test harness for algorithms; tests currently must be `#[cfg(test)]` in
  `src/` to run under `cargo test --bins`.
- Repository is not `cargo fmt`-clean; `clippy --bins` has pre-existing warnings.
- Hex rendering path (cell shape, projection, input mapping) undecided.
- Hex experiment now uses its own **flat-top** orientation and an
  aspect-preserving square viewport (mapping A unchanged). Mouse/selection input
  **is** hex-aware for the experiment (`hex::world_to_axial`); the square-grid
  input path is untouched. **Not visually inspected** — no screenshot tool in
  this environment; verified only at the "runs + geometry numerically correct"
  level.
- Whether to add `src/lib.rs` undecided (would fix `examples/`).
- **Hex model v0 blocker:** the boundary definition (single closest cell vs
  bracketing pair) and the sector **stop condition** are unresolved. The
  incremental recurrences are derived and exact, but the resulting cell set does
  not yet match a defensible reference. See research spec §11.6, §11.10.
- Model v0 passed a read-only reviewer pass; one over-claimed `[derived,
  verified]` sector statement was found and corrected (§11 revision note v0.1).

## Next concrete task

**Await user review/approval of research spec §11.** Experiment 1 (§12) —
hexagonal grid visualisation and reference classification — is implemented and
tested; the midpoint algorithm is **not**. After approval, resolve §11.6
(boundary definition + stop condition) using the experiment to compare candidate
definitions, then follow §11.9 (experiment-only code first, then an `Algorithm`).

Resolved previously and moved into §11: §8.1 (mappings A and B are the same
lattice up to `√3·R(−30°)`), §8.2 (integer `(x,y)` are axial hex coordinates;
their 6 nearest neighbours are exactly the axial moves at distance 1), §8.4
(`X²+Y² = x²+xy+y²`, exact).

## Exact verification commands to run

```sh
git status                                   # confirm no unexpected changes
cargo check                                  # typecheck (must pass)
cargo build                                  # build (must pass)
cargo test --bins                            # tests (22; must be ok)
cargo fmt --check                            # expected to FAIL (not fmt-clean)
cargo clippy --bins                          # expected warnings only
```

Do **not** run `cargo test` / `cargo test --all-targets` as a pass/fail gate
until `examples/test_benchmark.rs` is fixed.

## Do not touch (pre-existing user changes)

- `src/algorithms/ellipse/mod.rs` — modified (adds `own_decision_ellipse`).
- Untracked: `.devcontainer/`, `.github/`, `.gitignore`,
  `examples/`, `src/algorithms/ellipse/own_decision_ellipse.rs`,
  `test_benchmark_simple.rs`, `test_refactor.rs`.
