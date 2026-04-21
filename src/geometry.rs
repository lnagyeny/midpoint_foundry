use crate::algorithms::Algorithm;
use crate::renderer::Vertex;

// ── Low-level vertex helpers ──────────────────────────────────────────────────

/// Append a filled 1×1 unit-cell quad (2 triangles = 6 vertices).
#[inline]
fn push_quad(buf: &mut Vec<Vertex>, x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) {
    let (x1, y1) = (x + 1.0, y + 1.0);
    let v = |px: f32, py: f32| Vertex::new(px, py, r, g, b, a);
    buf.extend_from_slice(&[v(x, y), v(x1, y), v(x1, y1), v(x, y), v(x1, y1), v(x, y1)]);
}

/// Append a single line segment (2 vertices for LineList topology).
#[inline]
fn push_line(
    buf: &mut Vec<Vertex>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    buf.push(Vertex::new(x0, y0, r, g, b, a));
    buf.push(Vertex::new(x1, y1, r, g, b, a));
}

// ── Output ────────────────────────────────────────────────────────────────────

pub struct GeometryOutput {
    pub line_verts: Vec<Vertex>,
    pub quad_verts: Vec<Vertex>,
}

// ── Main entry point ──────────────────────────────────────────────────────────

pub fn build(
    algo: &dyn Algorithm,
    grid_size: i32,
    mouse_grid: Option<(f32, f32)>,
    selected_cell: Option<(i32, i32)>,
) -> GeometryOutput {
    let mut lines = Vec::<Vertex>::new();
    let mut quads = Vec::<Vertex>::new();

    let half = grid_size as f32 / 2.0;
    let imin = (-half) as i32;
    let imax = half as i32;

    // ── 0. Background fill for circle algorithms (white within radius) ────────
    // Simple approach: fill white circle for any algorithm that looks like a circle
    if let Some((cx, cy, r)) = algo.overlay_circle() {
        let r = r as f32;
        let cx = cx as f32;
        let cy = cy as f32;

        let start_x = (cx - r).floor() as i32;
        let end_x = (cx + r).ceil() as i32;
        let start_y = (cy - r).floor() as i32;
        let end_y = (cy + r).ceil() as i32;

        for x in start_x..=end_x {
            for y in start_y..=end_y {
                let fx = (x as f32 - cx) as f32;
                let fy = (y as f32 - cy) as f32;
                if fx * fx + fy * fy <= r * r {
                    push_quad(&mut quads, x as f32, y as f32, 1.0, 1.0, 1.0, 1.0);
                }
            }
        }
    }

    // ── 1. Algorithm points ───────────────────────────────────────────────────
    let color = algo.color();
    for (i, p) in algo.points().iter().enumerate() {
        let [r, g, b] = algo.point_color_override(i).unwrap_or(color);
        push_quad(&mut quads, p.x as f32, p.y as f32, r, g, b, 1.0);
    }

    // ── 2. Hover row/column highlights (soft blue, semi-transparent) ──────────
    if let Some((mx, my)) = mouse_grid {
        if mx >= -half && mx < half && my >= -half && my < half {
            let col = mx.floor() as i32;
            let row = my.floor() as i32;

            for r in imin..imax {
                push_quad(&mut quads, col as f32, r as f32, 0.5, 0.7, 1.0, 0.15);
            }
            for c in imin..imax {
                push_quad(&mut quads, c as f32, row as f32, 0.5, 0.7, 1.0, 0.15);
            }
        }
    }

    // ── 3. Selected-cell highlight (yellow, semi-transparent) ─────────────────
    if let Some((cx, cy)) = selected_cell {
        push_quad(&mut quads, cx as f32, cy as f32, 1.0, 1.0, 0.0, 0.45);
    }

    // ── 4. Grid lines (dark grey) ─────────────────────────────────────────────
    for i in imin..=imax {
        let f = i as f32;
        push_line(&mut lines, f, -half, f, half, 0.28, 0.28, 0.28, 1.0); // vertical
        push_line(&mut lines, -half, f, half, f, 0.28, 0.28, 0.28, 1.0); // horizontal
    }

    // ── 5. Coordinate axes (bright white) ─────────────────────────────────────
    push_line(&mut lines, -half, 0.0, half, 0.0, 1.0, 1.0, 1.0, 1.0); // X axis
    push_line(&mut lines, 0.0, -half, 0.0, half, 1.0, 1.0, 1.0, 1.0); // Y axis

    // ── 6. 45° diagonal in positive quadrant ──────────────────────────────────
    push_line(&mut lines, 0.0, 0.0, half, half, 1.0, 1.0, 1.0, 0.65);

    // ── 7. Analytic circle overlay (if algorithm provides one) ────────────────
    if let Some((cx, cy, radius)) = algo.overlay_circle() {
        let segments = 360usize;
        let ocx = cx as f32 + 0.5; // offset to pixel-centre convention
        let ocy = cy as f32 + 0.5;
        let r = radius as f32;

        for i in 0..segments {
            let a0 = 2.0 * std::f32::consts::PI * (i as f32) / segments as f32;
            let a1 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / segments as f32;
            push_line(
                &mut lines,
                ocx + r * a0.cos(),
                ocy + r * a0.sin(),
                ocx + r * a1.cos(),
                ocy + r * a1.sin(),
                1.0,
                1.0,
                1.0,
                1.0,
            );
        }
    }

    // ── 8. Analytic parabola overlay (if algorithm provides one) ─────────────
    if let Some((cx, cy, k_int)) = algo.overlay_parabola() {
        let ocx = cx as f32 + 0.5;
        let ocy = cy as f32 + 0.5;
        let k = 1.0 / k_int as f32;
        let y_max = (1.0 / (k * k) / 4.0).sqrt() as i32;

        for y in -y_max..=y_max {
            let x = (k * (y * y) as f32) as f32;
            if x.is_finite() {
                let y_f = y as f32;
                push_line(
                    &mut lines,
                    ocx - x,
                    ocy + y_f,
                    ocx + x,
                    ocy + y_f,
                    1.0,
                    1.0,
                    1.0,
                    1.0,
                );
            }
        }
    }

    // ── 9. Analytic ellipse overlay (if algorithm provides one) ─────────────
    if let Some((cx, cy, rx, ry)) = algo.overlay_ellipse() {
        let ocx = cx as f32 + 0.5;
        let ocy = cy as f32 + 0.5;
        let a = rx as f32;
        let b = ry as f32;
        let segments = 360usize;

        for i in 0..segments {
            let t0 = 2.0 * std::f32::consts::PI * (i as f32) / segments as f32;
            let t1 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / segments as f32;
            push_line(
                &mut lines,
                ocx + a * t0.cos(),
                ocy + b * t0.sin(),
                ocx + a * t1.cos(),
                ocy + b * t1.sin(),
                1.0,
                1.0,
                1.0,
                1.0,
            );
        }
    }

    GeometryOutput {
        line_verts: lines,
        quad_verts: quads,
    }
}
