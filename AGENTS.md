# AGENTS.md — Project instructions for AI-assisted work

Midpoint Foundry is a Rust desktop application for experimenting with,
visualising, and comparing **midpoint / Bresenham-style rasterisation
algorithms** (circles, ellipses, lines, parabolas) on a square pixel grid.
It was originally written in C++ and converted to Rust with AI assistance
under manual review.

These instructions are durable guidance for any AI coding session. They are
deliberately short. Detailed, evolving information lives in `docs/` and in the
three project skills under `.opencode/skills/`.

---

## 1. Read this first in a new session

1. Read `docs/ai/PROJECT_STATE.md` (short snapshot, one screen).
2. Read `docs/ai/SESSION_LOG.md` if you need history of what was already done.
3. Check `git status` and the current branch before touching anything.
4. Skim the source files you are about to change. Do not rely on memory.

---

## 2. Verified architecture (summary)

Full detail: `docs/architecture.md`.

- **Binary crate only.** `src/main.rs` builds a `winit` event loop and runs
  `app::Application`. There is **no `lib.rs`**, so `use midpoint_foundry::…`
  does not resolve. The only crate import path is `crate::…`.
- `src/app.rs` — window, egui/WGPU setup, algorithm registry, per-frame flow.
- `src/geometry.rs` — converts algorithm output into line/quad vertices.
- `src/renderer.rs` — WGPU pipelines, vertex buffers, orthographic transform.
- `src/shaders/shader.wgsl` — trivial vertex transform + flat colour fragment.
- `src/input.rs` — mouse → grid-coordinate mapping and cell selection.
- `src/ui.rs` — egui side panel (category/algorithm pickers, parameters,
  benchmark, selected-cell info).
- `src/benchmark.rs` — in-process timing of algorithms (UI-triggered).
- `src/algorithms/` — `mod.rs` defines `Point` and the `Algorithm` trait;
  `circle/`, `ellipse/`, `line/`, `parabola/` hold concrete algorithms and
  per-family base structs (`CircleBase`, `EllipseBase`).

Rendering is **not** a compute shader: algorithms run on the CPU and push
`Point { x: i32, y: i32 }` values. WGPU only draws the resulting GPU vertices.
See `docs/architecture.md` for the pipeline description.

---

## 3. Build, test, format, lint — actual commands

Run from the repository root. Verified against `cargo 1.96.0-nightly`.

```sh
cargo check                      # fastest full typecheck of the main binary
cargo build                      # build the application
cargo run                        # launch the app (needs a GPU/Vulkan-capable host)
cargo test --bins                # runs unit tests in the binary (22 tests: 17 in algorithms/hex + 5 in input; all pass)
cargo fmt                        # format source
cargo fmt --check                # verify formatting without writing
cargo clippy --bins              # lint the binary
```

Important caveats that are true today:

- `cargo test`, `cargo test --all-targets`, and `cargo check --all-targets`
  **fail**, because `examples/test_benchmark.rs` imports `midpoint_foundry::…`
  which does not exist (no library target). Use `cargo test --bins` instead.
- The repository is **not** `cargo fmt`-clean as committed. `cargo fmt --check`
  exits non-zero. Do not "fix" this as a drive-by change; if you reformat a
  file you are already editing, say so explicitly in the session log.
- `cargo clippy --bins` currently reports **6 warnings** (3 redundant `as f32` casts, 2 collapsible `if` statements, 1 function with too many arguments) at pre-existing locations in `geometry.rs` and `ui.rs`. These are pre-existing. Do not mass-fix them during unrelated work.
- `cargo test --bins` passes **22 tests** (17 in `src/algorithms/hex/`, 5 in `src/input.rs`). `cargo test` and `cargo test --all-targets` fail due to `examples/test_benchmark.rs` importing `midpoint_foundry::...` with no library target (use `cargo test --bins` instead).

---

## 4. Research process rules

These apply especially to the hexagonal-grid research module.

1. **Start substantial tasks by reviewing current state and the relevant
   source files.** Do not assume a file, type, or command exists.
2. **Agree on the model and derive the algorithm before implementing it.**
   For mathematical research, write down the coordinate system, the geometric
   definition of the shape, and the decision function first.
3. **Keep hexagonal-grid experiments isolated** from existing square-grid
   implementations. New code goes in its own module; do not edit the
   square-grid algorithms to make a hex experiment compile.
4. **Prefer the smallest change that tests a research hypothesis.**
5. **Do not refactor unrelated code** during research experiments.
6. **Do not introduce dependencies without a clear reason.** State the reason
   in the session log and in a decision record.
7. **Run the smallest relevant tests first**, then broader verification when
   appropriate.
8. **Do not silently alter existing algorithms or their expected behaviour.**
   If an existing algorithm has a bug, report it; fix it only in a separate,
   clearly labelled change.
9. **Update research documentation and the AI handoff** after meaningful
   completed work.
10. **Never commit, push, publish releases, or discard user changes** without
    explicit authorisation from the user.

### Mathematical correctness and reproducible experiments

- State the exact arithmetic domain (integer, fixed-point, floating point) and
  why it is sufficient.
- Prefer integer / incremental recurrences over recomputing an implicit
  function at every step when an exact recurrence exists.
- Compare generated cells against an explicitly defined reference (for example
  a brute-force predicate), not against visual appearance.
- Detect duplicate and missing cells programmatically.
- Report reproducible parameters (centre, radius, grid bounds, seed if any).

### Regression testing requirements

- Any change to an existing algorithm must keep its output identical unless
  the change is explicitly intended to alter it.
- Add a focused test for new functionality. If a test cannot be added yet,
  record why in the session log.
- A test may only be reported as passing if it was actually executed and
  passed. "Should pass" is not "passes".

---

## 5. Documentation and handoff requirements

- `docs/ai/PROJECT_STATE.md` — keep it current, keep it short. It must be
  readable at the start of every new session.
- `docs/ai/SESSION_LOG.md` — append-only chronological record. Add an entry
  after each meaningful completed task (objective, files, changes, commands,
  results including failures, findings, remaining work).
- `docs/ai/DECISIONS.md` — lightweight decision records for significant
  architectural or mathematical choices.
- `docs/architecture.md` — verified architecture; keep the verified/assumed
  distinction intact.
- `docs/research/hexagonal-grid.md` — the research specification. Read it
  before making mathematical changes.

Do not turn `AGENTS.md` into a manual. Do not turn `README.md` into a
development diary — keep it about the application and how to use it.

---

## 6. Skills to consult

Project-local skills live under `.opencode/skills/`. Load them with the
`skill` tool by their ID, or reference them as `@skill-id` in a prompt.

| Skill ID             | Use it when                                                        |
| -------------------- | ------------------------------------------------------------------ |
| `hexagonal-midpoint` | Designing, deriving, implementing, or validating hex midpoint work |
| `rust-wgpu-testing`  | Making Rust changes and verifying them in this repo                |
| `session-handoff`    | Starting or ending substantive work; maintaining handoff docs      |

An independent, read-only review agent is available at
`.opencode/agents/reviewer.md`. Ask the primary agent to
"use the reviewer subagent" to run it. It reports findings by severity and
does not modify files.

---

## 7. Restrictions

- Do not commit, push, tag, or publish without explicit authorisation.
- Do not discard, overwrite, or revert uncommitted user changes.
- Do not modify existing square-grid algorithm behaviour as a side effect.
- Do not add dependencies, plugins, global skills, or external automation
  without first explaining why they are needed and getting agreement.
- Do not report unverified results as verified. Distinguish measured facts,
  assumptions, and open questions.
- Do not claim a mathematically valid midpoint recurrence exists until it has
  been derived and checked.
