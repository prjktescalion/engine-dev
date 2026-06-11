// LCD panel post-process. The sprite pass renders segment "ink" into an
// offscreen target; this fullscreen pass composites it over a procedural
// LCD panel: greenish-grey base with a vertical gradient, a soft vignette,
// a 3px dot-matrix gutter grain, and posterization to a few dozen levels.
// This is where the Game & Watch look lives — segments themselves are just
// alpha masks.

struct Globals {
    resolution: vec2<f32>, // output size in pixels
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(1) var ink_tex: texture_2d<f32>;
@group(0) @binding(2) var ink_samp: sampler;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    // Fullscreen triangle.
    let corner = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    var out: VsOut;
    out.clip = vec4<f32>(corner * 2.0 - vec2<f32>(1.0, 1.0), 0.0, 1.0);
    out.uv = vec2<f32>(corner.x, 1.0 - corner.y);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let ink = textureSample(ink_tex, ink_samp, in.uv);

    // Panel base: greenish-grey, slightly darker toward the bottom.
    var col = mix(
        vec3<f32>(0.671, 0.702, 0.594),
        vec3<f32>(0.580, 0.616, 0.514),
        in.uv.y,
    );

    // Soft vignette.
    let d = distance(in.uv, vec2<f32>(0.5, 0.5));
    col *= 1.0 - 0.16 * smoothstep(0.35, 0.9, d);

    // Composite segment ink. The offscreen blend leaves premultiplied color;
    // recover the tint so overlapping segments don't darken twice.
    let tint = ink.rgb / max(ink.a, 1e-4);
    col = mix(col, tint, ink.a);

    // Dot-matrix grain: darken a 1px gutter every 3px.
    let cell = fract(in.clip.xy / 3.0);
    let gutter = max(step(cell.x, 0.34), step(cell.y, 0.34));
    col *= 1.0 - 0.030 * gutter;

    // Posterize — an LCD has few distinguishable levels.
    col = floor(col * 48.0 + vec3<f32>(0.5, 0.5, 0.5)) / 48.0;

    return vec4<f32>(col, 1.0);
}
