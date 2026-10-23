---
name: hexagonal-midpoint
description: Design, derive, implement, and validate midpoint-style (Bresenham-family) algorithms for hexagonal grids. Use for hex coordinate conventions, hex decision functions, incremental recurrences, and hex circle/curve rasterisation correctness. Do not use for square-grid pixel algorithms.
---

# Skill: hexagonal-midpoint

You are helping derive and validate **midpoint-style algorithms on hexagonal
grids**. These are correctness-critical mathematical procedures. Treat every
formula as a claim that must be derived and tested, never guessed.

## Non-negotiable first step

**Read `docs/research/hexagonal-grid.md` before making any mathematical
change.** It defines the objective, candidate coordinate systems, candidate
circle definitions, and the open questions. Also read
`docs/ai/PROJECT_STATE.md` for the current state and the next experiment.

If the research spec does not answer a question you need, **add the question to
it** rather than inventing an answer in code.

## Required workflow

### 1. Define the representation and convention explicitly

Before writing any algorithm, write down (in the research spec or the code's
doc comment):

- Is the target a **hexagonal cell grid** (cells, 6 neighbours each) or a
  **triangular/hexagonal point lattice** (points)? They are not the same.
- Which **coordinate system**: axial `(q, r)`, cube `(q, r, s)` with
  `q+r+s=0`, or offset (odd/even-r, odd/even-q)? State it and stick to it.
- Which **Cartesian mapping** turns coordinates into positions, whether
  hexagons are **pointy-top or flat-top**, and which direction is "direction 0".
- Whether the target shape is a **Euclidean disc** under that mapping or a
  **hex-metric cluster** (constant cube distance). These give different
  correct answers — do not silently pick one.

Never let a `(x, y)` pair mean two different things in different functions.

### 2. Derive the decision function from the geometric problem

Do not copy the square-grid recurrences. The classical `d += 2x + 3` is derived
for `F(X,Y) = X² + Y² − r²` under unit axis-aligned steps; hex steps are
different vectors and the midpoint offsets are different.

- Write the implicit function `F` for the chosen shape and mapping.
- Identify the candidate moves available from a cell in the sector you are
  traversing.
- Define the **midpoint** between the two candidate next cells.
- Evaluate the **sign** of `F` at that midpoint. If the midpoint has
  half-integer coordinates, scale `F` (e.g. evaluate `4F` at doubled
  coordinates) so the test stays exact in integers. State the scale.

### 3. Derive incremental updates algebraically

For a move `(dx, dy)`, expand `F(x+dx, y+dy) − F(x, y)` symbolically and show
it is **linear** in `x, y`. The **second difference** for a fixed move must be a
**constant**. Only then is an O(1) update valid.

- Show the algebra in a comment or in the research spec. Do not state an update
  formula you have not expanded.
- Prefer integer or exact fixed-point arithmetic. Justify why it is exact for
  all radii under consideration.
- **Do not recompute the full implicit function each step** when an exact
  incremental recurrence exists — that defeats the purpose of a midpoint
  algorithm. If you do recompute, justify it and say so.

### 4. Specify the full control flow

State, explicitly:

- **Candidate moves** in each sector, and how sectors are delimited.
- **Decision signs** — what `F(midpoint) < 0`, `= 0`, `> 0` mean here.
- **Tie-breaking** when the midpoint is exactly on the curve. Which move is
  taken, and does the rule preserve the grid's symmetry?
- **Initial value** of the decision variable at the start of the first sector.
- **Termination** condition and a bound on the number of steps. It must be
  provably finite and monotone where the classical `x ≤ y` was.
- **Symmetry / reflection** strategy for generating the rest of the boundary
  (which rotations/reflections, and whether the centre is on a cell).

### 5. Test smallest-first, against a reference

- Implement the **reference predicate** first (brute force over a bounded
  region) and confirm it independently — e.g. a hex ring of radius `r ≥ 1`
  should have `6r` cells and a filled cluster `1 + 3r(r+1)`. Verify these
  before trusting the reference to test the algorithm.
- For tiny radii (1…6), **exhaustively** compare algorithm output with the
  reference **cell by cell**. Do not move to larger radii until small ones pass.
- Compare against the explicit geometric reference, **not** against how the
  picture looks. Visual similarity is not correctness.
- Detect **duplicate** cells with a `HashSet` and **missing** cells by set
  difference against the reference — independently of rendering.
- Test **symmetry** by applying the grid's rotations/reflections to the output
  and asserting set equality.
- Record the exact parameters (centre, radius, bounds, arithmetic scale) needed
  to reproduce each result.

### 6. Report honestly

- Document **known limitations**: radii ranges tested, definitions assumed,
  cases not covered.
- **Never claim correctness without evidence.** "Looks right" and "should work"
  are not evidence.
- Distinguish what was *derived*, what was *tested*, and what remains an
  *open question*.
- If a derivation fails or a test fails, record the failure rather than
  weakening the test.

## Explicit prohibitions

- Do **not** force square-grid assumptions onto hex algorithms: no 90°/8-fold
  symmetry reasoning, no `octant` plotting, no `x ≤ y` termination copied
  blindly, no unit-axis steps assumed.
- Do **not** reuse `Point` from the square-grid code without establishing what
  its coordinates mean for hex.
- Do **not** edit existing square-grid algorithms to make hex code compile.
- Do **not** introduce dependencies without justification.
- Do **not** mark the research complete, or commit an algorithm, until the
  reference comparison and the symmetry/duplicate/coverage checks pass.

## Hand-off

After meaningful progress, update `docs/ai/SESSION_LOG.md` and
`docs/ai/PROJECT_STATE.md`, and move resolved questions out of
`docs/research/hexagonal-grid.md` §8 into derived facts. Use the
`session-handoff` skill pattern.
