// Instanced sprite batch: one 4-vertex triangle strip per instance, expanded
// in the vertex shader (no vertex buffer). Instances carry a design-space
// rect, an atlas UV rect, and an RGBA tint; the atlas is R8 coverage, so the
// fragment output is tint * coverage — standard alpha blending does the rest.

struct Globals {
    // Design-space viewport size (sprites are authored in these units).
    viewport: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var atlas_tex: texture_2d<f32>;
@group(0) @binding(2) var atlas_samp: sampler;

struct Instance {
    @location(0) pos: vec2<f32>,    // center, design units
    @location(1) size: vec2<f32>,   // full extent, design units
    @location(2) uv_min: vec2<f32>,
    @location(3) uv_max: vec2<f32>,
    @location(4) color: vec4<f32>,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, inst: Instance) -> VsOut {
    // vi 0..3 -> (0,0) (1,0) (0,1) (1,1), drawn as a strip.
    let corner = vec2<f32>(f32(vi & 1u), f32(vi >> 1u));
    let world = inst.pos + (corner - vec2<f32>(0.5, 0.5)) * inst.size;
    let ndc = world / globals.viewport * 2.0 - vec2<f32>(1.0, 1.0);

    var out: VsOut;
    out.clip = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0); // y-down design space
    out.uv = mix(inst.uv_min, inst.uv_max, corner);
    out.color = inst.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let coverage = textureSample(atlas_tex, atlas_samp, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * coverage);
}
