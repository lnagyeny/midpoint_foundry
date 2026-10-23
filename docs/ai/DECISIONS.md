# Decision records

Lightweight ADR-style log. One record per significant architectural or
research decision. Do not duplicate implementation detail from the session log.

Format:

```
## DR-NNN — <title>
Date:
Status: proposed | accepted | superseded by DR-MMM
Context:
Decision:
Alternatives considered:
Consequences:
```

---

## DR-001 — Use OpenCode-native project-local skills

**Date:** 2026-10-22
**Status:** accepted

**Context:** Future AI sessions need reusable, version-controlled instructions
that match the repository, and the project should stay self-contained.

**Decision:** Put skills in `.opencode/skills/<id>/SKILL.md` using the native
V2 format (frontmatter `name`, `description`). Keep skills scoped to this
repository; do not install global skills or plugins.

**Alternatives considered:** Global `~/.config/opencode/skills` (not versioned
with the repo, not shared with collaborators); `AGENTS.md`-only (too coarse —
would bloat project instructions); a `skills` config array pointing elsewhere
(unnecessary indirection).

**Consequences:** Skills are discovered automatically per project. IDs are
directory-derived (`hexagonal-midpoint`, `rust-wgpu-testing`,
`session-handoff`). Any collaborator running OpenCode in this repo gets the
same instructions.

---

## DR-002 — Read-only reviewer subagent, no orchestration

**Date:** 2026-10-22
**Status:** accepted

**Context:** Independent review is valuable, but parallel automatic code
modification is risky for correctness-sensitive mathematical work.

**Decision:** Add `.opencode/agents/reviewer.md` as a `mode: subagent` agent
with `edit` and `shell` denied, so it can only read and report findings by
severity. First priority is a reliable single-agent workflow plus an
independent review option. No multi-agent orchestration is introduced.

**Alternatives considered:** A primary-mode reviewer (competes with the build
agent, easy to run by accident); a plugin-based multi-agent orchestrator
(adds a dependency and complexity before the workflow is proven).

**Consequences:** The reviewer cannot fix what it finds; fixes are made by the
primary agent. `cargo`/test commands cannot be run *by* the reviewer, so it
reviews diffs and sources rather than execution results.

---

## DR-003 — Keep hex research isolated; defer the `lib.rs` decision

**Date:** 2026-10-22
**Status:** accepted

**Context:** The crate is binary-only, so `examples/` and root scratch files do
not compile, and there is no place for integration tests. Adding `src/lib.rs`
would fix this but is a structural change to a working project.

**Decision:** Do not add `src/lib.rs` as part of the workflow setup. For now,
new hex code goes in its own isolated module tree, and new tests are written as
`#[cfg(test)] mod tests` inside `src/` so they run under `cargo test --bins`.
Leave the existing square-grid algorithms untouched. Revisit the `lib.rs`
question separately when a hex module needs a real test target or an example.

**Alternatives considered:** Add `src/lib.rs` immediately (fixes `examples/`
but changes the crate shape mid-setup and requires touching `main.rs`/imports);
a separate workspace member crate (heavier, premature).

**Consequences:** `cargo test` and `cargo test --all-targets` remain broken and
must not be used as pass/fail gates. `cargo test --bins` is the supported test
command. `examples/test_benchmark.rs` stays broken until this is revisited.

---

## DR-004 — Treat `x² + xy + y² = r²` as a candidate, not a specification

**Date:** 2026-10-22
**Status:** accepted

**Context:** A previously considered mapping `X = (√3/2)x, Y = y + x/2` yields
the quadratic `x² + xy + y² = r²` for a Euclidean circle. It is tempting to
treat this as the definition of the hex circle.

**Decision:** Record it in `docs/research/hexagonal-grid.md` as a *candidate*
implicit function whose correctness depends on (a) what integer `(x, y)`
denote, (b) whether the mapping matches the intended grid, and (c) whether the
target shape is the Euclidean disc or the hex-metric cluster. Require these to
be established by experiment before use. Do not claim any midpoint recurrence
exists until derived and checked.

**Alternatives considered:** Adopting the quadratic outright (risks encoding an
unverified convention into code and tests).

**Consequences:** The first research task is a lattice-identity check, not an
algorithm. `PROJECT_STATE.md` and the `hexagonal-midpoint` skill both require
reading the research spec before mathematical changes.

---

## DR-005 — No new dependencies or external automation

**Date:** 2026-10-22
**Status:** accepted

**Context:** The setup asks for a reliable workflow using native features.

**Decision:** Use only native OpenCode configuration (project skills, a
subagent) and the existing Rust toolchain. Add no dependencies, plugins, global
skills, CI automation, or external services.

**Alternatives considered:** Adding `criterion` for benchmarks (already listed
transitively in `Cargo.lock` but not a declared dev-dependency; unnecessary
now); adding `proptest` for property tests (defer until the hex module needs
it, and justify it then).

**Consequences:** Verification relies on `cargo` and manual discovery checks.
If a future experiment genuinely needs a dependency, it must be justified in
the session log and a new decision record.

---

## DR-006 — Hex model v0 uses mapping A as the canonical centre lattice

**Date:** 2026-10-22
**Status:** proposed (awaiting user review of research spec §11)

**Context:** §8.1 and §8.2 asked whether mapping A
(`X=(√3/2)x, Y=y+x/2`) and the standard axial→pixel mapping B describe the same
lattice, and what integer `(x, y)` denote. A scratch computation (outside the
repo) established that `B = √3·R(−30°)·A` and that the 6 nearest integer
neighbours under mapping A are exactly the axial hex moves at distance 1. The
Euclidean circle becomes `x² + xy + y² = r²` exactly.

**Decision (proposed):** Adopt mapping A with integer `(x, y)` as **axial hex
coordinates** as the canonical convention for the first hex experiments, and
target the **Euclidean** circle (definition 2) on cell centres. Treat mapping B
as an equivalent display choice only.

**Alternatives considered:** Adopting mapping B as canonical (equivalent, but
`x²+xy+y²` is the natural form under A, and A relates directly to the
previously considered convention); targeting the **hex-metric** ring
(definition 1) instead (simple and fully symmetric, but not a Euclidean circle,
so it does not exercise the midpoint/quadratic derivation).

**Consequences:** The first sector spans polar angles **30°→90°** under mapping
A (not 0°→60°), which is unusual and is the likely source of the unresolved
sector/stop problem in §11.6. This 30° offset must be handled explicitly.
Changing this decision later would invalidate the derived decision variable and
recurrences.

---

## DR-007 — Model v0 is presented, not implemented; §11.6 blocks coding

**Date:** 2026-10-22
**Status:** accepted

**Context:** Deriving the hex midpoint model surfaced a real mismatch between the
naive midpoint walk and a defensible boundary reference, plus an underived stop
condition.

**Decision:** Record the model in `docs/research/hexagonal-grid.md` §11 as a
**proposal for review**, clearly label the blocking item, and **write no source
code** until the user approves the model and §11.6 is resolved. The later first
code step must be experiment-only (pure functions + tests), with no `Algorithm`
implementation and no changes to square-grid code.

**Alternatives considered:** Implementing the walk now and iterating in code
(would encode an unverified boundary definition and risk confounding rendering
with math, which the project process forbids).

**Consequences:** The research document now distinguishes derived facts,
observations, and open questions. No `Algorithm` struct exists for hex; the
registry is unchanged.

---

## DR-008 — Hex experiment isolated via a defaulted `overlay_geometry` trait method

**Date:** 2026-10-22
**Status:** accepted

**Context:** Experiment 1 requires drawing flat-top hexagon cells, which the
square-grid geometry path (`geometry.rs::push_quad`) cannot express. Changing
that path risks altering the 16 existing square-grid algorithms' rendering.

**Decision:** Add one **optional, defaulted** trait method to `Algorithm`:

```rust
fn overlay_geometry(&self) -> Option<crate::algorithms::hex::ExperimentGeometry> { None }
```

`geometry::build` consults it **first**; when `Some`, it returns the supplied
vertices verbatim and skips all square-grid drawing. All existing algorithms
inherit the `None` default, so their behaviour is byte-for-byte unchanged. The
hex research code lives entirely in the new module `src/algorithms/hex/`.

**Alternatives considered:** (a) A separate hex geometry builder invoked from
`geometry::build` by category string — noisier, and couples `geometry.rs` to the
hex module by name; (b) a second executable/example — would leave the app UI
unavailable and duplicate the window/renderer setup; (c) reusing `push_quad`
with skewed quads — cannot produce hexagons.

**Consequences:** One trait method and one guarded early-return are the only
changes to existing files. The `Algorithm` trait now names the hex experiment's
geometry type; if the experiment is removed, the method and type go with it.
Input mapping (mouse → cell) is still square-grid-based and is **not** hex-aware;
that is a known limitation, not addressed here.

---

## DR-009 — Aspect correction is a defaulted `viewport()` override, not a change to mapping A

**Date:** 2026-10-22
**Status:** accepted

**Context:** The hex experiment looked diagonally skewed because the original
projection maps the world square onto the full *rectangular* grid area, giving x
and y different pixel scales. Fixing this must not change the mathematical
coordinate system (mapping A, `F = x²+xy+y²`), the neighbour definitions, or the
meaning of radius. It must also not affect the 16 square-grid algorithms.

**Decision:** Add a second **optional, defaulted** trait method:

```rust
fn viewport(&self) -> Option<(f32, f32, f32)> { None }   // (cx, cy, half)
```

When `Some`, `renderer::render` (via a small `renderer::GridView` descriptor)
maps the world square `[c−half, c+half]²` to a **square** physical viewport of
side `min(area_width, height)`, centred horizontally, using a single uniform
scale on both axes (`Uniforms::ortho_centered(cx, cy, half)`). The default
`None` keeps the original projection and full-area viewport, which reduce
byte-for-byte to the previous `Uniforms::ortho(half)` + `set_viewport(0,0,w,h)`.

Mouse input is inverted with a matching `input::ScreenMap`; when the override is
present the cursor is converted to an axial cell via `hex::world_to_axial`
(cube rounding), so hover/selection are hex-aware.

**Alternatives considered:** (a) Anisotropic scale on the world→NDC transform
(e.g. multiply x by the aspect ratio) — simplest, but it makes the drawing
non-conformal: hexagons would be stretched and the "ideal circle" would render
as an ellipse, defeating the experiment's purpose; (b) forcing a square *window*
— intrusive, changes the app for all algorithms; (c) fixing only the circle by
rescaling it — cosmetic and misleading.

**Consequences:** Hexagons stay regular and the ideal circle stays circular at
any window shape, at the cost of letterboxing when the grid area is not square.
Mapping A, `F`, the neighbours and the radius are untouched. The default `None`
leaves every existing algorithm unchanged. A future non-hex viewport override
would currently be routed through the hex hit test; this coupling is named
explicitly by `input::is_hex_experiment` and should be generalised if a second
override is ever added.

---

## DR-010 — Remove the two never-used `EllipseBase` helpers instead of allowing dead code

**Date:** 2026-10-22
**Status:** accepted

**Context:** `cargo check`/`clippy` reported dead code for
`EllipseBase::ring_color` and `EllipseBase::draw_ui_with_duplicates`.

**Decision:** **Remove** both methods (no `#[allow(dead_code)]`).

**Evidence:** `git log -S` shows both were introduced in commit `ae995bb` and
never called since. Every ring-colouring ellipse algorithm
(`gap_fill_ellipse`, `own_fill_ellipse`, `own_decision_ellipse`) inlines
`PALETTE_7[r % 7]` instead of calling `ring_color`, and all ellipse algorithms
delegate their UI to `draw_ui_full` (which contains the duplicates checkbox), so
`draw_ui_with_duplicates` has no user. No branch or stashed work references them
(`git grep` over `main`, `origin/main`, `origin/dev`), and no documentation
describes them as intended API.

**Alternatives considered:** Restoring a call site (would change the algorithms'
current colouring/UI behaviour for no functional gain); `#[allow(dead_code)]`
(hides a genuine obsolescence signal).

**Consequences:** `clippy --bins` warning locations drop from 9 to 6, none in
the touched files; no behaviour changes. `src/algorithms/ellipse/mod.rs`
(pre-existing user edit adding `own_decision_ellipse`) was not touched.
