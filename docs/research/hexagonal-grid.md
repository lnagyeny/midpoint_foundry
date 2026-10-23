# Research specification: hexagonal-grid midpoints

**Status: draft specification + first proposed model (see §11). No algorithm has
been implemented. The model in §11 is a proposal for review, not a verified
result.**

This document defines the problem precisely enough to begin work, records what
is known, and lists what must be resolved *before* any implementation. Nothing
here should be read as a claim that a correct midpoint recurrence for hex grids
is known. §11 records the first concrete proposed model, including an
*unresolved* mismatch between the naive midpoint walk and the "closest cell"
reference that must be settled before implementation.

Read this file before making mathematical changes (see the
`hexagonal-midpoint` skill).

---

## 1. Research objective

Produce and validate **midpoint-style (Bresenham-family) algorithms for
hexagonal grids**: incremental, low-cost cell-generation procedures that
produce a hexagonal-grid approximation of a circle (and, later, other conics),
in the spirit of the classical midpoint circle algorithm on a square lattice.

Success criteria (to be refined, but the intent is fixed):

- The generated set of hex cells approximates an explicitly defined geometric
  reference within a stated tolerance.
- The procedure is incremental: per-step work is O(1) additions/comparisons
  with integer (or exact fixed-point) arithmetic, not a re-evaluation of the
  implicit function at every cell.
- Symmetry, connectivity, coverage, duplicate-freedom, and termination are
  stated and verified, not assumed.
- The result is reproducible from documented parameters.

Out of scope for the first experiment: filled regions, GPU compute, anti-
aliasing, and multi-shape generalisation.

---

## 2. Hexagonal **cell grid** vs triangular / hexagonal **point lattice**

This distinction must be settled before any coordinate code is written.
Confusing the two is the most likely source of a wrong "circle".

- A **hexagonal cell grid (hex tiling):** the plane (or a region) is partitioned
  into hexagonal cells. Each cell has a centre, and the adjacency graph is a
  triangular lattice: every cell has up to 6 neighbours. A "circle" is a set of
  *cells* whose centres approximate a circle (or whose cells approximate a
  disc). This is the natural analogue of "pixels on a square grid".

- A **triangular lattice of points:** sample points arranged so that each point
  has 6 equidistant nearest neighbours. This is the dual description: the
  centres of a hex tiling form a triangular lattice.

- A **hexagonal lattice of points:** a sparser lattice where each point has
  6 nearest neighbours at equal distance and the next shell has 12, etc.
  (Not usually what "hex grid" means in rasterisation.)

**Decision needed:** the first experiment rasterises *which* of these? The
working assumption is **hexagonal cell grid** (cells indexed by hex
coordinates; each cell drawn as a hexagon), because that is the direct
analogue of the existing square-grid pixel work. This is an **assumption to be
confirmed**, not a fact.

**Consequence for rendering:** the existing app stores `Point { x, y }` and
draws unit squares. Hex cells would need either a hexagon-fill per cell or an
oblique projection trick (see §5 and `docs/architecture.md` §9–10).

---

## 3. Candidate coordinate systems

For a hex cell grid, three standard systems are candidates. Which one the
project uses must be stated explicitly and used consistently.

Let the six neighbour directions be numbered consistently.

### 3.1 Axial coordinates `(q, r)`
Two integers. A third is implied: `s = −q − r`. Neighbours:
`(±1,0), (0,±1), (+1,−1), (−1,+1)`.

### 3.2 Cube coordinates `(q, r, s)` with `q + r + s = 0`
Redundant but fully symmetric. Useful for deriving symmetric decision
functions and symmetry tests, because all six directions are group-equivalent
under permutation and sign. Often the best system for *proving* symmetry, then
projected to axial for storage.

### 3.3 Offset coordinates (odd-r / even-r / odd-q / even-q)
Rectangular array with rows shifted by half a cell. Convenient for array
storage and for the classic oblique mapping (see §4), but the adjacency rules
depend on row parity, which makes symmetry reasoning error-prone.

**Working assumption:** derive in **cube coordinates** (symmetry and distance
are simple), store in **axial** (`Point`-like 2-int struct, or a dedicated
`HexPoint { q, r }`). Offset only if a rectangular buffer is required.

Cube distance: `max(|Δq|, |Δr|, |Δs|) = (|Δq| + |Δr| + |Δs|) / 2`.

Axial ↔ cube: `q = q`, `r = r`, `s = −q − r`.

**[unresolved]** Confirm the neighbour ordering and the sign convention used
for the "hex circle": which of the 6 directions is defined as direction 0, and
in which screen orientation.

---

## 4. Discrete coordinates ↔ Cartesian geometry

A hex cell coordinate must map to a screen position. Two conventions are in
play and **they are not obviously the same grid**.

### 4.1 Candidate mapping A — the `X = (√3/2)x, Y = y + x/2` mapping

Previously considered in this project:

```
X = (√3 / 2) · x
Y = y + x / 2
```

Interpretation: `(x, y)` are integer grid coordinates; `(X, Y)` is the
Cartesian position. This is a linear (affine, shear) transform. The shear
`Y = y + x/2` is exactly what turns a square `(x, y)` lattice into a
triangular point lattice: unit steps in `x` move by `(√3/2, 1/2)`, unit steps
in `y` move by `(0, 1)`, and those two vectors have equal length and meet at
60°. So mapping A describes a **generator pair for a triangular lattice**
(i.e. it can represent hex neighbours or the centres of a hex tiling).

Under mapping A, a Euclidean circle
`X² + Y² = r²` becomes

```
(√3/2)² x² + (y + x/2)² = r²
(3/4) x² + y² + x y + (1/4) x² = r²
x² + x y + y² = r²
```

**[verified by algebra]** the quadratic form is `x² + xy + y²`. This form is
positive definite; its level sets are ellipses in the `(x, y)` plane and
circles in `(X, Y)`. Its matrix `[[1, 1/2], [1/2, 1]]` has determinant `3/4`.

**Must be established, not assumed:**
- Does mapping A correspond to the intended hex grid (cell centres) or to a
  different (offset/sheared square) arrangement?
- Is `(x, y)` meant to be axial coordinates, offset coordinates, or the
  *generators* of the lattice?
- Critically: **is the intended shape a circle with respect to Euclidean
  distance in `(X, Y)`, or a hex-metric circle (`max` of cube deltas / ring of
  constant cube distance)?** These are different sets of cells and yield
  different "hex circles". The quadratic `x² + xy + y² = r²` is only correct
  for the *Euclidean* interpretation under mapping A.

### 4.2 Candidate mapping B — standard axial ↔ pixel

The more common hex convention (Red Blob Games style), with cell size `s`:

```
X = s · (√3 · q + (√3/2) · r)
Y = s · (3/2 · r)
```
(or a 60°-rotated variant). This makes the six neighbours of a cell all at
distance `s·√3` in `(X, Y)`, which is the property mapping A also has when
`(x, y)` are understood as the two generators.

**[unresolved]** Show explicitly whether mapping A and mapping B describe the
*same* set of cell centres up to a rotation/scale, or genuinely different
lattices. This should be settled with a small numeric check, not by intuition.

### 4.3 Other Cartesian conventions worth naming

- **Oblique offset grid:** draw a shear directly as a WGPU transform and keep
  drawing unit quads. Cheapest visually; the "cell" is a parallelogram/sheared
  square, not a hexagon. Acceptable for early experiments but not a true hex
  cell shape.
- **Pointy-top vs flat-top hexagons:** changes the neighbour directions and the
  pixel mapping. Must be chosen and recorded.

---

## 5. Candidate definitions of "circle" and its boundary on a hex grid

Several inequivalent definitions exist. **Choosing one is a prerequisite.**
Each gives a different set of cells and therefore a different correct answer.

1. **Star / shadow / "hex circle" in hex metric.** Cells within cube distance
   `r` of the centre. The boundary is the set of cells at exactly distance `r`.
   This produces a hexagon-shaped cluster, not a Euclidean-looking circle.
   Simple and fully symmetric; a good **reference implementation**.

2. **Euclidean disc on cell centres.** Cells whose *centre* (under a chosen
   Cartesian mapping) satisfies `X² + Y² ≤ r²` (filled) or lies within a band
   around `X² + Y² = r²` (outline). Depends on the mapping in §4. This is what
   the quadratic `x² + xy + y² = r²` describes under mapping A.

3. **Cell-based coverage.** A cell is "inside" if the cell *region* (not just
   its centre) intersects the disc, or if the cell centre is the nearest cell
   to the ideal circle. Rasterisation-rule choices multiply.

4. **Threshold/offset variants of 2** (e.g. `|X² + Y² − r²| < ε(r)` rather than
   `≤ 0`), which determine the outline's thickness and symmetry.

**Working assumption:** for the *first* experiment, define the reference as
definition 2 on cell centres under the chosen mapping, and treat definition 1
as a separate, independently verifiable object (exact ring counts, known
formulas) used to sanity-check symmetry and adjacency code.

**[unresolved]** What makes a hex-grid "circle" *look* circular is not the same
as what makes it *correct* under a metric. State explicitly which is being
pursued.

Known combinatorial facts worth using as tests (verify, don't trust blindly):
a hex ring of radius `r ≥ 1` in hex metric contains `6r` cells; the filled
cluster contains `1 + 3r(r+1)` cells. These give cheap exact assertions.

---

## 6. Candidate midpoint-style decision functions and incremental updates

No valid recurrence has been derived yet. This section states the *form* the
derivation must take and records a candidate starting point.

### 6.1 What "midpoint" means

The classical midpoint circle algorithm works in the first octant because of
8-fold symmetry. At each step it tests the midpoint between the two candidate
next pixels against the implicit function `F(X, Y) = X² + Y² − r²`, keeps an
integer decision variable, and updates it by a *second difference* that depends
only on the current `x` or `y`. The key property: `F` is a quadratic form, so
its discrete differences are linear, hence exactly representable in integers.

### 6.2 Candidate starting point (mapping A / Euclidean circle)

Under mapping A the implicit function is

```
F(x, y) = x² + x y + y² − r²
```

(integer-valued for integer `x, y`). Candidate direction of travel: along a
hex "first sector" of the boundary, stepping between two neighbouring hex
directions.

**Must be derived explicitly (this is the actual work):**

- **Decision function at a midpoint.** For candidate moves `m1`, `m2` from the
  current cell, the test is `sign F(cell + (m1 + m2)/2)`. Because the midpoint
  may be half-integer, choose a scaled integer form (e.g. `4F` on doubled
  coordinates) so the test is exact. *Which* midpoint and *which* two moves
  depends on the sector geometry — this has to be drawn out.
- **Initial value.** `F` (or its scaled form) at the first cell of the sector.
- **Increments.** Derive `ΔF` for each allowed move algebraically:
  `F(x + dx, y + dy) − F(x, y)` is linear in `x, y`:
  `Δ = (2dx + dy)·x + (dx + 2dy)·y + (dx² + dx·dy + dy²)`.
  The *second* difference (change of `Δ` between consecutive steps) is a
  constant for a fixed move, which is what makes an O(1) update possible.
- **Termination.** The classical octant loop ends at `x > y`. The hex analogue
  is "the sector has been traversed", i.e. the direction has rotated by 60°;
  the exact stopping condition must be derived and must be provably finite.
- **Tie-breaking.** When `F(midpoint) = 0` the two candidates are equidistant;
  the rule (which move is taken) must be specified and shown to preserve
  symmetry, otherwise the result will be asymmetric.

### 6.3 Alternative: hex-metric midpoint

If the target is definition 1 (§5), the "decision function" is `max(|q|,|r|,|s|) − r`,
which is not a smooth quadratic and has no second-difference recurrence of the
classical kind. Any "midpoint" algorithm for the hex metric must be justified
differently. **Do not assume the classical derivation carries over.**

### 6.4 Explicit non-claims

- No correct hex midpoint recurrence is known at time of writing.
- `x² + xy + y² = r²` is a candidate **implicit function** under a candidate
  mapping; it is not yet an algorithm, and it is not yet established that it
  describes the intended cells.
- The classical square-grid recurrences (`d += 2x + 3`, etc.) **must not** be
  copied into hex code; they are derived for `X² + Y² − r²` on unit steps.

---

## 7. Requirements the eventual algorithm must satisfy

State each as a testable property.

- **Symmetry.** The generated set must be invariant under the hex grid's
  symmetry group (6-fold rotations and reflections about the centre), for a
  centre placed on a cell. For other centres, state the reduced symmetry.
- **Connectivity.** Adjacent generated cells should be hex-neighbours, or a
  documented larger step; no unexplained jumps and no isolated cells.
- **Coverage.** Every cell that the chosen definition (§5) places on the
  boundary is generated; no boundary cell is missed. Verify against the
  reference predicate over a bounded region.
- **No duplicates.** Unless duplicates are intentional and documented (some
  existing square algorithms deliberately emit duplicates), the outline must
  contain each cell at most once. Duplicate detection must be programmatic
  (a `HashSet`), not visual.
- **Termination.** The loop must provably terminate after a bounded number of
  steps. State the bound.
- **Determinism / reproducibility.** Same inputs → identical output; all
  parameters recorded.
- **Arithmetic.** State whether the decision variable is integer, fixed-point
  (and its scale), or floating point, and why it is exact for all radii tested.
- **No full re-evaluation.** Per-step work must be O(1) with no recomputation
  of the implicit function (that is the whole point of a midpoint algorithm);
  any deviation must be justified and documented.

---

## 8. Open mathematical questions

1. Does mapping A (§4.1) and mapping B (§4.2) generate the same cell centres?
   Under what rotation/scale/reflection?
2. Under mapping A, what do integer `(x, y)` denote — axial hex coordinates,
   offset coordinates, or lattice generators? Does the answer change which
   cells are "neighbours"?
3. Is the intended hex "circle" the Euclidean disc (definition 2) or the
   hex-metric cluster (definition 1)? They are different; both are legitimate.
4. Is the quadratic form `x² + xy + y²` the correct implicit function for the
   chosen mapping and definition? Derive it from the mapping rather than
   assuming it.
5. How many distinct "sectors" does the hex boundary split into, and what are
   the allowed move pairs in each? Is the split the same as the 6-fold
   rotational sectors, or finer (as the square case splits 90° into 45°
   octants)?
6. Does a midpoint decision function on hex moves have an exact integer form,
   or is a scale factor (e.g. ×2 or ×4) unavoidable? What is the smallest such
   scale?
7. Is the resulting boundary symmetric under all 6 rotations, or only under a
   subgroup, for a given tie-breaking rule? Which tie rules preserve symmetry?
8. For definition 1, is there a *useful* incremental algorithm at all beyond
   direct ring enumeration, and what would "midpoint-style" even mean?
9. How does a hex outline compare, cell-for-cell, to the square-grid midpoint
   circle mapped through the same lattice? (Useful cross-check, not the goal.)
10. What is the analogue of the classical `x ≤ y` stopping condition, and is it
    reached monotonically?

---

## 9. Experiments needed before implementation

Ordered, smallest-first. Each should be runnable as a test that prints or
asserts concrete numbers.

1. **Lattice sanity check.** Implement mapping A and mapping B as pure
   functions; for the first few cells, print Cartesian positions and nearest-
   neighbour distances. Confirm which pairs are equidistant and thus which
   points are genuinely hex neighbours. Resolve §8.1–8.2.
2. **Reference predicates.** Implement, as brute force over a bounded region:
   (a) the hex-metric ring/disc (definition 1); (b) the Euclidean predicate
   `x² + xy + y² ≤ r²` on centres (definition 2). Confirm (a) against the
   closed-form counts `6r` and `1 + 3r(r+1)`.
3. **Sector mapping.** For a mid-size radius, colour/label each boundary cell
   by the direction from which it was reached and publish the sector
   decomposition. Resolve §8.5.
4. **Difference table.** Numerically compute `F(cell + move) − F(cell)` and the
   second differences for each candidate move; confirm they are linear/constant
   as the algebra in §6.2 predicts, using exact integers.
5. **Tiny-radius exhaustive check.** For radii 1…6, enumerate every cell and
   compare candidate algorithm output with the reference predicate cell by
   cell. Look for missing/duplicate/asymmetric cells. Resolve §8.6–8.7.
6. **Symmetry test.** Apply all 6 rotations/reflections to the generated set
   and assert set equality with itself for a centre at the origin.
7. **Cross-check.** Map the square-grid `MidpointCircle` output through the
   lattice and diff against the hex result to understand the discrepancy.

Only after experiments 1–5 succeed should any algorithm be committed as
"implemented".

---

## 10. Terminology to fix before coding

| Term | Meaning to fix |
| ---- | -------------- |
| cell | one hexagonal tile; the unit of rasterisation |
| cell centre | the point used for distance tests |
| axial / cube / offset | the three coordinate conventions (§3) |
| direction 0…5 | the six neighbour moves, with a fixed numbering and orientation |
| sector | a maximal arc of the boundary traversed by one move rule |
| hex metric | cube distance `(‖Δq‖+‖Δr‖+‖Δs‖)/2` |
| Euclidean reference | distance in the chosen `(X, Y)` mapping |
| decision variable | the integer value tested each step |
| reference predicate | the brute-force definition the algorithm is compared to |

---

## 11. PROPOSED MODEL v0 — first hexagonal midpoint design (for review)

**Status: PROPOSAL, not verified. Do not implement until the unresolved item in
§11.6 is resolved.** Every numerical claim below was checked with a scratch
calculation (Python, exact rationals where relevant) outside the repository; no
repository source was changed. Where a claim is derived it is marked
**[derived]**; where it is only observed it is marked **[observed]**; open items
are marked **[open]**.

**Revision note (v0.1).** A read-only reviewer pass found one **false claim**
carrying a `[derived, verified]` tag (the §11.3 "only `{NE, NW}` advance the
angle" statement — corrected below), plus several smaller labelling and range
errors. The corrections are applied in place and listed here:

- §11.3: the monotone-angle justification is **withdrawn and replaced** by an
  explicit `dy = +1` **marching convention** (verified counterexamples included).
- §11.4: added the integer/`i32` **arithmetic bound** (`4r²` must fit, so
  `r ≤ 2¹⁴`) and clarified the sign rule outside the circle.
- §11.6: rewrote the stop-condition item; `x == 0` **is** the 90° end, the real
  problem is the overshoot (`(0,7)` at `r = 6`) and the missing stop invariant.
- §11.7: relabelled the angle table as a **target-angle** table and added an
  **`NE`-branch worked example at `r = 4`** (the `r = 3` case only uses `NW`).
- §11.8: widened the decision-variable test range to `0..=r+1`.
- §11.10 Q13: **closed** — the rotation `(x,y) → (x+y, −x)` suffices.

### 11.1 What is being modelled (cell grid vs lattice) — CONFIRMED

- The target is a **hexagonal cell grid**: the unit of rasterisation is a hex
  *cell*, and the output is a set of cells. Each cell has a centre; the
  adjacency graph is triangular (6 neighbours).
- The centres of a hex tiling form a **triangular lattice**. So "hex cell grid"
  and "triangular lattice of centres" are two views of the same object.
- Mapping A (§4.1) indexes exactly that centre lattice. **[derived — see §11.2]**
- A "point lattice" in the sense of §2 (a sparse lattice whose shells have
  6, 12, … points) is **not** what we rasterise; it is a different object.

### 11.2 Coordinate system and Cartesian geometry — RESOLVED

**Convention adopted for the proposal.**

- Integer coordinates `(x, y)` are **axial hex coordinates** (the two generators
  of the centre lattice). Cube coordinates are `(q, r, s) = (x, y, −x−y)`.
- Cartesian mapping (mapping A):
  `X = (√3/2)·x`, `Y = y + x/2`. **[adopted]**

**Derived facts (checked numerically, exact up to floating error):**

1. **The 6 nearest integer neighbours of the origin under mapping A are exactly
   the 6 axial hex directions** `(±1,0), (0,±1), (+1,−1), (−1,+1)`, all at
   Cartesian distance **1**. So the integer lattice under mapping A *is* the
   hex-centre lattice, and the axial move set is exactly the nearest-neighbour
   set. **[derived, verified]**
2. **Mapping A and mapping B describe the same lattice.** With
   `B(q,r) = (√3·q + (√3/2)·r, (3/2)·r)`, we have
   `B = M · A` where `M = √3 · R(−30°)`, i.e. a rotation by −30° composed with
   a scale of `√3`. `MᵀM = 3·I`, `det M = 3`. **[derived, verified** on a
   sample of points to 1e−9**]**. This resolves §8.1: they are the *same*
   lattice up to a rigid motion and scale, so the choice is only a display
   convention, not a different geometry.
3. **Euclidean circle ⇒ quadratic form.** `X² + Y² = x² + xy + y²`
   for integer `(x, y)`. **[derived, verified]** (`F(x,y) := x² + xy + y²`.)

**Which "circle"?** For the proposal the target is definition 2 (§5):
the Euclidean unit-distance circle in `(X, Y)`, i.e. level sets of `F`. This is
a **proposal**; definition 1 (hex-metric ring) is **not** the same set and is
used only as an independent structural sanity check.

### 11.3 Sector and candidate moves — CONVENTION ADOPTED (correction applied)

Under mapping A the point `(r, 0)` lies at polar angle **30°**, and `(0, r)` at
**90°** (not 0° and 60°). **[derived, verified]** So the natural first arc is the
**60° arc from 30° to 90°**, with both endpoints on lattice points.

**Correction (was wrong in an earlier draft).** An earlier version of this
section claimed `{NE, NW}` are the *only* moves that increase polar angle
monotonically, and marked it `[derived, verified]`. **That claim is false.** Away
from `(r,0)` the move `W = (−1, 0)` also advances the polar angle. Verified
counterexamples (mapping A angles):

| cell | θ(cell) | move | target | θ(target) | Δθ | F |
| ---- | ------- | ---- | ------ | --------- | -- | - |
| (5,1) | 38.9° | W | (4,1) | 40.9° | **+1.9°** | 21 |
| (5,2) | 46.1° | W | (4,2) | 49.1° | **+3.0°** | 28 |
| (4,3) | 55.3° | W | (3,3) | 60.0° | **+4.7°** | 27 |
| (3,0) | 30.0° | W | (2,0) | 30.0° | 0.0° | 4 |

So on the segment `x > y` the angle can also be advanced by `W`, which moves
*inward* (`F` decreases). This is exactly the geometry that produced the
over-thick walk in §11.6: the "advance by one angular step" operation is not
uniquely `{NE, NW}`.

**What is actually established here** is only a convention, which we adopt
explicitly:

> **Adopted marching convention (proposed).** Traverse the first sector from
> `(r,0)` to `(0,r)` using a **`dy = +1` monotone staircase**: at each cell the
> *only* choice is between `NE = (0,+1)` (outer, `F` grows) and
> `NW = (−1,+1)` (inner, `F` shrinks). The row index `y` increases by exactly 1
> each step, so the walk has exactly one cell per `y` and terminates when `y`
> reaches its stop value.

This is a **definitional choice**, not a derived consequence of "monotone
angle", and the earlier angle-based justification is withdrawn. Whether this
convention yields a defensible outline is precisely the unresolved question in
§11.6.

By 60°-rotational symmetry the other five sectors are images of this one. The
rotation is the integer map

```
(x, y) → (x + y, −x)          [verified: F is invariant, F(x+y,−x) = F(x,y)]
```

which is a genuine 60° lattice rotation about the origin that preserves the
quadratic form `F`. **[derived, verified]** An earlier draft worried that the
30°-offset might force a *reflection* instead; that worry is **withdrawn**: the
rotation above closes the ring without any reflection (see open question 13).
**[proposed]**

**Only one sector needs to be derived by hand; the rest follow by rotation.**

### 11.4 Decision function — DERIVED

Candidate moves from cell `(x, y)`: `NE → (x, y+1)` and `NW → (x−1, y+1)`.
The **midpoint of the two targets** is `(x − 1/2, y + 1)`.

Exact expansion **[derived step by step]**:

```
F(x, y)                 = x² + xy + y²
F(x − 1/2, y + 1)       = (x−1/2)² + (x−1/2)(y+1) + (y+1)²
                        = x² + xy + y²  +  0·x  +  (3/2)·y  +  3/4
                        = F(x, y) + (3/2)·y + 3/4
```

Multiply by 4 to clear the halves and define the **integer decision variable**:

```
D(x, y) := 4·F(x, y) + 6·y + 3 − 4·r²          [integer for integer x,y,r]
```

`D` is the scaled value of the implicit function at the midpoint, relative to
the circle:

- `D > 0` ⟺ midpoint is **outside** the circle ⟹ the curve passes *inside* the
  segment ⟹ take the **inner** target ⟹ **`NW`**.
- `D ≤ 0` ⟺ midpoint is **inside** or on ⟹ take the **outer** target ⟹ **`NE`**.

**[derived]**; the sign rule was corrected once during derivation (an initial
inverted-sign attempt produced a degenerate walk — see §11.6). The sign rule is
only meaningful while the current cell is inside or on the circle; once a walk
step lands strictly outside (as happens for `r = 4`, §11.7), "inner/outer
target" must be read as "the target with smaller/larger `F`", not as
"inside/outside the circle".

**Arithmetic domain.** All quantities in this section are **integer**. For
`i32`, `4·r²` must fit, so the safe bound is `r ≤ 2¹⁴` (`4·r² < 2³¹`);
`r = 2¹⁵` already overflows `i32` in the `−4r²` term. `|D| = O(r²)`, so the
whole recurrence is comfortably representable in `i32` for all radii the
renderer can plausibly show. **[derived, verified]**

### 11.5 Incremental recurrence — DERIVED and verified

For a move `(dx, dy)`, the exact first difference is

```
ΔF = (2dx + dy)·x + (dx + 2dy)·y + (dx² + dx·dy + dy²)
```

Hence, at a fixed cell, the change in `D` for each move is linear in `x, y`:

| Move | Target | `ΔD` |
| ---- | ------ | ---- |
| `NE` | `(x, y+1)` | `4x + 8y + 10` |
| `NW` | `(x−1, y+1)` | `−4x + 4y + 10` |

**[derived; each verified against the direct formula on a grid of small
integers].** Because these increments are linear, we can carry them as
auxiliary integer variables `s := 4x + 8y + 10` (NE step) and
`t := −4x + 4y + 10` (NW step) and update them by constants:

- after `NE` (`y += 1`): `s += 8`, `t += 4`;
- after `NW` (`x −= 1, y += 1`): `s += 4`, `t += 8`.

So each step costs a few integer additions and one comparison — the required
O(1) behaviour, with **no re-evaluation of `F`**. **[derived]**.

Initialisation at the sector start `(r, 0)`:

```
D₀ = 4r² + 0 + 3 − 4r² = 3
s₀ = 4r + 8·0 + 10     = 4r + 10
t₀ = −4r + 0 + 10      = 10 − 4r
```

**[derived, verified]** (`D(r,0) = 3` for all `r` — a convenient constant.)

### 11.6 THE UNRESOLVED ITEM (blocking) — [open]

The incremental walk above produces, for `x`-stop at `x = 0`:

```
r = 6:  (6,0) (5,1) (5,2) (4,3) (3,4) (2,5) (1,6) (0,7)
```

(Only `r = 6` is reproduced here because it was directly checked; other radii
were computed but the sector-stop rule differs between attempts, so no other
sequence is quoted as fact.)

Two problems were observed and are **not yet resolved**:

1. **Stop condition.** The `dy = +1` convention (§11.3) terminates on a *count*,
   not on a geometric invariant, and the count is currently chosen ad hoc.
   `x == 0` **is** the 90° sector end (verified: `(0,r)` is exactly at polar
   angle 90°), so the real issue is not "is `x==0` the boundary" but that the
   walk overshoots `x = 0` before stopping: for `r = 6` the walk's last cell is
   `(0,7)`, i.e. `y = r+1 > r`, and there is no `dy`-based invariant that says
   "stop at `x = 0`" (because the walk can take an `NW` step from `(1, y)` to
   `(0, y+1)`, skipping the column `x = 0` entirely). A **principled termination
   rule** (the analogue of the classical `x ≤ y`) has **not** been derived.
   **[open]**
2. **The walk does not equal every reasonable "closest cell" reference.**
   Comparing the walk against a brute-force reference defined as *"for each `y`,
   the cell whose centre minimises |F − r²|"*, the sets differ for several
   radii (`r = 1…8` checked). The discrepancy is systematic, not random: the
   walk sometimes takes **both** an `x`-step and a `y`-step where the reference
   takes one. This means **either** the reference definition is the wrong one
   (the two bracketing cells are both defensible), **or** the midpoint rule /
   segment geometry is still mis-specified. **This must be settled before any
   implementation.** **[open]**

**Withdrawn in this revision:** the earlier claim that "`x == 0` is *not* a 60°
sector boundary". `(0,r)` *is* the 90° end of the arc. The unresolved part is
reference mismatch and the lack of a stop invariant, as stated above.

**Honest summary:** the coordinate system, the implicit function, the candidate
move pair, the decision variable, and the incremental recurrences are derived
and internally consistent. The *matching of the resulting cell set to a
defensible boundary definition*, and the *principled termination*, are **not**
yet established. There is **no** verified hex midpoint algorithm yet.

### 11.7 Worked example (small radius, hand-checkable)

Take `r = 3`, start `(3, 0)`.

```
Step 0: cell (3,0).   D = 3   (constant start).   3 > 0 → NW.
Step 1: → (2,1).  D += t₀ = 10 − 4·3 = −2  ⇒ D = 1.   Update s,t: s=4·2+8·1+10=26, t=−4·2+4·1+10=6.
        1 > 0 → NW.
Step 2: → (1,2).  D += 6 ⇒ D = 7.  7 > 0 → NW.
Step 3: → (0,3).  x == 0 ⇒ stop.
Sector: (3,0) (2,1) (1,2) (0,3)
```

Cross-check against the direct formula `D(x,y) = 4F(x,y) + 6y + 3 − 36`:
`D(3,0)=3`, `D(2,1)=4·7+6+3−36=1`, `D(1,2)=4·7+12+3−36=7`. **[verified]**
So `r = 3` gives a clean **pure diagonal** `(3,0)→(2,1)→(1,2)→(0,3)`, i.e.
`(x,y) = (r−k, k)`. **Note:** `r = 3` takes the **`NW` branch only**, so it does
not exercise the `NE` branch at all.

**Second worked example, exercising the `NE` branch: `r = 4`.** Start `(4,0)`,
`D₀ = 3`, `s₀ = 4·4+10 = 26`, `t₀ = 10 − 16 = −6`.

```
Step 0: cell (4,0).  D = 3 > 0 → NW.
        D += t₀ = −6  ⇒ D = −3.   New cell (3,1).
        After NW (x−=1,y+=1): s += 4 → 30, t += 8 → 2.
Step 1: cell (3,1).  D = −3 ≤ 0 → NE.
        D += s = 30  ⇒ D = 27.    New cell (3,2).
        After NE (y+=1): s += 8 → 38, t += 4 → 6.
Step 2: cell (3,2).  D = 27 > 0 → NW.
        D += t = 6   ⇒ D = 33.    New cell (2,3).
        After NW: s += 4 → 42, t += 8 → 14.
Step 3: cell (2,3).  D = 33 > 0 → NW.
        D += t = 14  ⇒ D = 47.    New cell (1,4).
        ... (continues; stop rule still ad hoc, §11.6).
Sector (with the current x==0 stop): (4,0) (3,1) (3,2) (2,3) (1,4) (0,5)
```

**Verified against the direct formula** `D(x,y) = 4F(x,y) + 6y + 3 − 64`:

| cell | F | `D` (direct) | matches incremental |
| ---- | - | ------------ | ------------------- |
| (4,0) | 16 | 3 | ✔ |
| (3,1) | 13 | −3 | ✔ |
| (3,2) | 19 | 27 | ✔ |
| (2,3) | 19 | 33 | ✔ |
| (1,4) | 21 | 47 | ✔ |

Note that `D` becomes **negative** at step 1 (`(3,1)`, `F = 13 < 16`): the walk
briefly lands strictly *inside* the circle, then jumps outside. This is why the
sign rule is stated in terms of "smaller/larger `F`" in §11.4. **[derived,
verified]**

Move-angle check at `(3,0)` (mapping A polar angles of the **targets**):

| move | target | angle° | Δangle° | F |
| ---- | ------ | ------ | ------- | - |
| E | (4,0) | 30.0 | 0.0 | 16 |
| NE | (3,1) | 37.6 | +7.6 | 13 |
| NW | (2,1) | 38.9 | +8.9 | 7 |
| W | (2,0) | 30.0 | 0.0 | 4 |
| SW | (3,−1) | 21.1 | −8.9 | 7 |
| SE | (4,−1) | 22.4 | −7.6 | 13 |

At `(3,0)` the table selects `{NE, NW}` — but this is a **single-cell**
coincidence, and as shown in §11.3 the choice does **not** generalise. The
adopted `dy = +1` convention is therefore a definition, not a consequence of
this table.

### 11.8 Required correctness tests (automatically checkable)

These are the properties to encode as tests; they are stated so that a test can
fail honestly.

1. **Lattice identity (experiment 1).** `B(q,r) ≈ √3·R(−30°)·A(q,r)` for a
   sample of coordinates, tolerance 1e−9. Also: the 6 axial moves from the
   origin all have Cartesian length 1.
2. **Quadratic form.** `X² + Y² == x² + xy + y²` exactly for integer `(x,y)`
   (using integer arithmetic on a doubled/scaled representation).
3. **Decision variable.** `D(x,y,r) == 4·F(x,y) + 6y + 3 − 4r²` and the
   incremental updates reproduce `D` directly, for all `x,y` in `0..=r+1`
   (the upper bound matters: the walk can produce `y = r+1`, e.g. `(0,7)` at
   `r = 6`, so a `0..r` range would not cover the produced cells).
4. **Reference predicate.** Brute force `F(x,y) ≤ r²` over a bounded region;
   independently confirm the hex-*metric* ring counts `6r` and disc `1+3r(r+1)`
   (definition 1) as a sanity check on the adjacency code.
5. **Tiny-radius exhaustive comparison.** For `r = 1…6`, compare the generated
   outline to the chosen reference **cell by cell** (set equality, both
   directions). *Currently this comparison FAILS against the naive reference —
   do not implement until §11.6 is resolved.*
6. **Symmetry.** Apply all 6 rotations generated by `(x,y) → (x+y, −x)` to the
   full ring and assert set equality with itself (centre at the origin). No
   reflection is required — the rotation preserves `F` (§11.3).
7. **No duplicates.** Assert the generated ring has no repeated cell
   (`HashSet` length == vector length).
8. **Coverage.** Assert every reference boundary cell appears.
9. **Termination.** Assert the loop terminates within a stated bound (e.g.
   `≤ r + 2` steps per sector) for `r = 1…N`.
10. **Determinism.** Same inputs give an identical, ordered cell list.

Because there is no `lib.rs` (see `docs/architecture.md` §7), these tests must
live in a `#[cfg(test)] mod tests` block and run via `cargo test --bins`.

### 11.9 Smallest implementation plan (after approval)

**Do not start until §11.6 is resolved and the model is approved.**

- **Step 1 — experiments only, no `Algorithm` impl.** Add one isolated module
  `src/algorithms/hex/mod.rs` (plus a `pub mod hex;` line in
  `src/algorithms/mod.rs`) containing *pure functions only*:
  `mapping_a`, `cube_rotate`, `F`, `D`, the brute-force reference, and the
  tests from §11.8. No rendering, no `Algorithm` trait, no registry entry yet.
- **Step 2 — resolve §11.6.** Use the tests to decide the boundary definition
  and the stop condition. Record the decision in `docs/ai/DECISIONS.md`.
- **Step 3 — minimal algorithm struct.** Only if steps 1–2 pass: add a
  `HexMidpointCircle` implementing `Algorithm`, with `Point { x, y }` holding
  **axial** coordinates, and a clearly documented note that these are *not*
  square-grid pixels. Register it under a new category `"hex"`.
- **Step 4 — rendering.** Deferred. The square-grid renderer draws unit
  squares; a hex cell needs a polygon emitter or an oblique projection. This is
  a separate, later task and must not block the math.
- **Do not** modify any existing square-grid algorithm, `Point`'s meaning, or
  the registry ordering as part of this work.

### 11.10 New open questions raised by v0

11. Is the correct boundary the **single closest cell** per angular step, or
    the **pair of cells bracketing** the curve (a thicker outline)? Both are
    standard; the Euclidean hex outline is not uniquely defined without this
    choice. **[open]**
12. What is the **principled sector stop condition**, and is it reached
    monotonically? Does the `30°`-offset of mapping A force the "sector" to be
    30°–90° (a 60° arc but not aligned to the axes)? **[open]**
13. ~~Does one sector rotate into the next by a pure cube rotation, or is a
    reflection required?~~ **CLOSED.** The integer map `(x,y) → (x+y, −x)` is a
    genuine 60° lattice rotation that preserves `F` (verified). Rotation alone
    closes the ring; no reflection is needed.
14. Should the decision test be at the midpoint of the two **targets**, or at
    the midpoint of the **step between representative points** of the cells
    (a Voronoi/duality question)? **[open]**
15. Does the classical-style "keep an auxiliary increment" form remain exact in
    integer arithmetic for all radii, or is a scale >4 needed in some sectors?
    **[open]**
16. Is there any hex analogue of the classical `x ≤ y` that is provably
    monotone, or must termination be defined by counting sector steps? **[open]**

---

## 12. EXPERIMENT 1 — hexagonal grid visualisation and reference classification

**Status: IMPLEMENTED and TESTED (visualisation + reference only). No midpoint
algorithm. Boundary definition still unresolved.**

This experiment makes §11's geometry visible so that candidate boundary
definitions can be compared before a final midpoint rule is adopted. It does
**not** implement the incremental walk described in §11.4–§11.5 as an algorithm.

### 12.1 Isolation strategy (how square-grid rendering is preserved)

The square renderer hard-codes unit-square cells (`geometry.rs::push_quad`) and
square grid lines, so hexagons cannot be drawn through it. Instead of changing
that path, two **optional, defaulted** trait methods were added:

```rust
// src/algorithms/mod.rs, `Algorithm`
fn overlay_geometry(
    &self,
    _mouse_cell: Option<(i32, i32)>,
    _selected_cell: Option<(i32, i32)>,
) -> Option<crate::algorithms::hex::ExperimentGeometry> {
    None
}

fn viewport(&self) -> Option<(f32, f32, f32)> {   // (cx, cy, half)
    None
}
```

`geometry::build` now checks `overlay_geometry` **first** and, if `Some`, returns
the supplied vertices verbatim (skipping every square-grid wisp). `Renderer::render`
consults `viewport` (via a small `GridView` descriptor): when `Some`, it maps the
world square to a **square** physical viewport with a single uniform scale, so
hexagons stay regular and circles stay circular (letterboxed otherwise). Every
existing algorithm inherits both `None` defaults, so their rendering is
byte-for-byte unchanged. **[verified: `ortho_centered(0,0,half)` and the default
viewport `(0,0,area_w,height)` reduce exactly to the previous `ortho(half)` +
`set_viewport(0,0,w,h)`]**

Files changed:

| File | Change |
| ---- | ------ |
| `src/algorithms/mod.rs` | `pub mod hex;` + defaulted `overlay_geometry` and `viewport` |
| `src/geometry.rs` | early-return guard at the top of `build` |
| `src/renderer.rs` | `ortho_centered` + `GridView` square-viewport resolution |
| `src/input.rs` | `ScreenMap` transform + hex-aware cursor→axial routing |
| `src/app.rs` | `GridView` construction + one `HexGridExperiment` registry entry |
| `src/algorithms/hex/mod.rs` | **new**, isolated research module |

No change to `ui.rs`, `shader.wgsl`, or any square-grid algorithm. The category
`"hex"` is discovered automatically by the existing UI (`ui.rs` builds categories
dynamically).

### 12.2 Coordinate mapping and geometry

- Axial `(x, y)` → Cartesian via mapping A: `X = (√3/2)·x`, `Y = y + x/2`
  (`hex::axial_to_cartesian`). Because the screen layout *is* mapping A, the
  ideal circle `F = r²` renders as a **true Euclidean circle**. **[unchanged by
  the orientation work: mapping A, `F`, neighbours and radius keep their exact
  meaning]**
- **Aspect correction is a render/input concern only**, decoupled from the maths
  through the `viewport()` override (DR-009). The screen orientation is
  **flat-top**: neighbour centre directions are at 30°/90°/150°/…, shared edges
  are the flat sides. The default `None` view keeps the original square-grid
  projection.
- Six neighbour offsets `NEIGHBOURS` = `(1,0),(0,1),(−1,1),(−1,0),(0,−1),(1,−1)`,
  all at Cartesian distance 1. **[tested]**
- **Flat-top hexagons** with circumradius `R = 1/√3 = √3/3` (apothem `1/2`).
  Adjacent hexagons therefore share edges exactly, with no gaps or overlaps.
  **[tested: `adjacent_hexagons_share_edges`]**
- The drawn region is the **world square** `[−H, H]²` (field `world_half`); the
  visible square is one unit larger (`viewport` renders `[−(H+1), H+1]²`) so edge
  hexagons are clipped rather than ending on the boundary. `region_cells` covers
  the square with no gaps. **[tested: `region_covers_world_square` — every sample
  point's nearest axial cell is present and each sample lies inside its covering
  hexagon; exhaustive in the discrete domain]**
- Mouse hit test: `hex::world_to_axial` uses cube rounding (Red Blob rule) to pick
  the nearest cell centre; `input.rs` routes the experiment's hover/selection
  through it. **[tested: `world_to_axial_recovers_cell`,
  `world_to_axial_matches_nearest_centre` (brute-force reference)]**
- Fills are emitted as a **6-triangle fan with a repeated centre vertex** (18
  vertices per cell) so there are no internal seams and no index buffer is
  needed. Outlines are 6 independent edges (12 vertices); hover/selection fills
  are painted **before** outlines so no darkened shared edges appear. **[tested]**

### 12.3 Reference classification (NOT a boundary)

Each cell centre is classified by its explicit relationship to the circle
(`hex::CellClass`):

| class | rule | colour |
| ----- | ---- | ------ |
| `Inside` | `F < r²` | blue |
| `OnCircle` | `F == r²` | white |
| `NearOutside` | `0 < F − r² ≤ r` | amber |
| `FarOutside` | `F − r² > r` | dim grey |

The `≤ r` threshold is a **display choice** (a band about *half* a lattice step
thick in `F`; one lattice step changes `F` by about `2r`); it is not derived
from any midpoint rule. **No cell is labelled "the rasterised boundary".** The
purpose is to see, visually, where the two defensible boundary definitions
(§11.10 Q11) would differ. **[documented, not a claim]**

### 12.4 Tests actually executed

`cargo test --bins` → **22 passed, 0 failed**:

```
emitted_fill_forms_closed_hexagon          ok
circle_outline_has_requested_radius        ok
geometry_vertex_counts                     ok
hexagon_geometry_is_correct                ok
adjacent_hexagons_share_edges              ok
neighbours_are_at_unit_distance            ok
neighbour_offsets_are_distinct_and_hex_distance_one  ok
quadratic_form_identity_holds              ok
f_quadratic_matches_naive                  ok
rotate60_preserves_f_and_has_order_six     ok
classification_boundaries                  ok
on_circle_cells_have_exact_f               ok
cartesian_axial_round_trip                 ok
world_to_axial_recovers_cell               ok
world_to_axial_matches_nearest_centre      ok
hover_and_selection_add_highlights         ok
region_covers_world_square                 ok
region_cells_have_no_duplicates            ok
region_is_closed_under_neighbour_moves_in_interior  ok
default_map_round_trips_square_grid        ok
experiment_viewport_is_square_and_centred  ok
experiment_viewport_is_isotropic           ok
```

The quadratic-form identity is checked to `1e−3` floating tolerance over
`x,y ∈ [−12,12]`; `f_quadratic` is checked to be exactly equal to the naive
integer polynomial over `[−50,50]²`.

### 12.5 Commands run and results

- `cargo check` → succeeds; **0 warnings**.
- `cargo test --bins` → **22 passed / 0 failed**.
- `cargo clippy --bins` → **6 warnings**, all at pre-existing locations
  (`geometry.rs` 16/83/84/121/186, `ui.rs` 51). **Down from the 9-warning
  baseline**; none in touched files. The `EllipseBase` dead-code warning is gone
  (removed in this session).
- `cargo build` → succeeds.
- `cargo run` → **the application launched** on the software Vulkan adapter
  (`llvmpipe`) and ran without code panicking (verified ≥ 8 s). The
  `Event loop failed: ExitFailure(1)` / Broken pipe teardown occurs **only** on
  SIGTERM/`timeout` and is **identical in the pre-change baseline binary**
  (rebuilt and re-run to confirm) — pre-existing teardown noise. **The window was
  not visually inspected** (no screenshot tool available: `grim`/`spectacle`/
  `import`/`scrot` all absent), so the flat-top orientation and aspect-correct
  square filling are verified at the level of *"renders + geometry numerically
  correct"* **not** "pixels were seen". This must be confirmed by the user.

### 12.6 Arithmetic typo check (requested)

The "initial decision-variable auxiliary value" arithmetic was re-checked in
§11.5: `s₀ = 4r + 8·0 + 10 = 4r + 10` and `t₀ = 10 − 4r` are **correct**; `D₀ = 3`
for all `r`. The `r = 4` worked example in §11.7 was replayed against the direct
formula and matches exactly. **No typo remains in the research documentation**;
the earlier `8y + 10` slip existed only in scratch output and was never
committed.

### 12.7 Remaining mathematical questions

Unchanged and still blocking a final algorithm (§11.6, §11.10):

1. **Boundary definition** — single closest cell per angular step, or the pair
   of cells bracketing the curve? Q11.
2. **Sector termination** — a principled, monotone stop invariant (the analogue
   of classical `x ≤ y`); the current `x == 0` stop overshoots (Q12, Q16).
3. **Decision-test placement** — midpoint of the two targets vs the step
   midpoint (Q14).

This experiment exists precisely so these can be compared visually.
