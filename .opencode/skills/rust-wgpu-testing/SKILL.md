---
name: rust-wgpu-testing
description: Guide safe Rust implementation and verification in this repository, including builds, tests, and WGPU/egui rendering changes. Use when editing Rust code in midpoint_foundry, adding tests, or verifying rendering behaviour.
---

# Skill: rust-wgpu-testing

Use this when making **Rust changes in this repository** and verifying them.
Read `docs/architecture.md` first if you have not already.

## Understand the actual architecture before changing it

- The crate is a **binary only** — there is **no `src/lib.rs`**, so
  `use midpoint_foundry::…` does not resolve. Internal code uses `crate::…`.
- **Algorithms run on the CPU.** WGSL is only used for the final vertex
  transform and a flat-colour fragment shader. Do **not** assume an algorithm
  runs on the GPU just because the project uses WGSL — it does not.
- Data flows: `Algorithm::compute()` (CPU) → `Vec<Point>` →
  `geometry::build` → line/quad `Vertex` buffers → `Renderer::render` (WGPU)
  plus an egui pass.
- The algorithm registry is a `Vec<Box<dyn Algorithm>>` in `src/app.rs`.
  Adding an algorithm = new file + a `pub mod` line + a `Box::new(...)` push.
- `Point { x: i32, y: i32 }` names a cell, drawn as the unit square
  `[(x,y), (x+1,y+1)]`. See `docs/architecture.md` §4.

## Rules for the change itself

1. **Follow the existing architecture and conventions.** Match surrounding
   naming, module layout, and `Algorithm` trait usage. Do not invent a new
   pattern when one already exists.
2. **Inspect existing implementations before introducing abstractions.** If a
   helper looks needed, check `CircleBase`, `EllipseBase`, `geometry.rs`,
   and `ui.rs` first — it may already exist.
3. **Prefer small, isolated changes.** One hypothesis per change. Keep hex
   work in its own module and out of the square-grid files.
4. **Add focused tests for new functionality.** Because there is no `lib.rs`
   and no `tests/` directory, put tests in a `#[cfg(test)] mod tests` block
   inside the module you changed, so they run with `cargo test --bins`.
   Do not claim a test passes unless you actually ran it.
5. **Separate algorithmic correctness from visualization correctness.** A
   correct cell set can look wrong and a wrong one can look fine. Test the
   algorithm on its output (`points()`), not on the rendered image.
6. **Preserve existing square-grid behaviour.** Do not change an existing
   algorithm's output, its defaults, colours, or UI unless that is the
   explicit task. If you find a bug, report it and fix it in a separate,
   labelled change.
7. **Avoid unnecessary dependencies and premature refactoring.** No new crates
   without a stated reason. Do not restructure unrelated code during a research
   experiment.
8. **Report commands that could not be run and why.** Sandbox/display/GPU
   limitations are normal; say so instead of implying success.

## Verification commands (run the smallest relevant first)

```sh
cargo check                  # fastest typecheck of the binary — run this first
cargo test --bins            # run unit tests (currently 0 tests)
cargo build                  # full build
cargo clippy --bins          # lint the binary (pre-existing warnings exist)
cargo fmt --check            # formatting check
```

Known, **pre-existing** conditions (do not "fix" them as drive-by changes):

- `cargo fmt --check` **fails** — the committed tree is not fmt-clean. If you
  format a file you are already editing, mention it explicitly.
- `cargo clippy --bins` emits warnings (collapsible `if`, redundant `as f32`,
  too many arguments, dead code). Leave unrelated ones alone.
- `cargo test`, `cargo test --all-targets`, and `cargo check --all-targets`
  **fail** because `examples/test_benchmark.rs` imports a nonexistent library
  crate. Do not use them as gates; use `cargo test --bins`.

## Verifying rendering changes

- Launching the app needs a working GPU/Vulkan surface and a display. If you
  cannot launch it, say so and explain what you verified instead.
- For rendering changes, state clearly which of these you did:
  (a) changed geometry generation and verified `Vertex` output numerically,
  (b) ran the application and observed it, or (c) only compiled.
- Do not describe a rendering change as "verified" if only (a) or (c) happened.
- The benchmark (`src/benchmark.rs`) is an in-process timer reachable from the
  UI; it is not a test. A timing difference is not proof of algorithmic
  correctness. Do not make performance claims without measured numbers and
  the exact command/parameters.

## Hand-off

If the change is meaningful, append to `docs/ai/SESSION_LOG.md` with real
results (including failures) and update `docs/ai/PROJECT_STATE.md`. Use the
`session-handoff` skill.
