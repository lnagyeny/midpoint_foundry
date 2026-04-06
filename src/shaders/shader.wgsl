// ── Uniforms ──────────────────────────────────────────────────────────────────
//
// A simple orthographic transform: maps grid coordinates (-half … +half) to
// NDC clip space (-1 … +1).  The matrix is diagonal so column-major layout
// doesn't matter in practice.

struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// ── Vertex ────────────────────────────────────────────────────────────────────

struct VertexIn {
    @location(0) position : vec2<f32>,
    @location(1) color    : vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_pos : vec4<f32>,
    @location(0)       color    : vec4<f32>,
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_pos = uniforms.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    out.color    = in.color;
    return out;
}

// ── Fragment ──────────────────────────────────────────────────────────────────

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}
