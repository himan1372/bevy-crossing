// N64 colour combiner: (a - b) * c + d, one or two cycles.
// Mux numbering follows the RDP (see n64_material.rs).

#import bevy_pbr::forward_io::VertexOutput

struct Combiner {
    color0: vec4<u32>,
    alpha0: vec4<u32>,
    color1: vec4<u32>,
    alpha1: vec4<u32>,
    prim: vec4<f32>,
    env: vec4<f32>,
    extra: vec4<f32>,
    flags: u32,
};

const FLAG_TWO_CYCLE: u32 = 1u;
const FLAG_LIT: u32 = 2u;
const FLAG_TEXTURED: u32 = 4u;
const FLAG_NO_COMBINER: u32 = 8u;
const FLAG_NO_SHADE: u32 = 16u;

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> cc: Combiner;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var tex_sampler: sampler;

struct Inputs {
    combined: vec4<f32>,
    texel: vec4<f32>,
    shade: vec4<f32>,
};

// Inputs shared by colour a, b and d (index < 6).
fn common_rgb(sel: u32, i: Inputs) -> vec3<f32> {
    switch sel {
        case 0u: { return i.combined.rgb; }
        case 1u, 2u: { return i.texel.rgb; }
        case 3u: { return cc.prim.rgb; }
        case 4u: { return i.shade.rgb; }
        case 5u: { return cc.env.rgb; }
        default: { return vec3(0.0); }
    }
}

fn color_a(sel: u32, i: Inputs) -> vec3<f32> {
    if sel == 6u { return vec3(1.0); }
    if sel == 7u { return vec3(0.5); } // noise
    return common_rgb(sel, i);
}

fn color_b(sel: u32, i: Inputs) -> vec3<f32> {
    return common_rgb(sel, i); // 6 center, 7 K4, 8+ zero
}

fn color_c(sel: u32, i: Inputs) -> vec3<f32> {
    switch sel {
        case 7u: { return vec3(i.combined.a); }
        case 8u, 9u: { return vec3(i.texel.a); }
        case 10u: { return vec3(cc.prim.a); }
        case 11u: { return vec3(i.shade.a); }
        case 12u: { return vec3(cc.env.a); }
        case 14u: { return vec3(cc.extra.x); } // primitive LOD fraction
        default: { return common_rgb(sel, i); }
    }
}

fn color_d(sel: u32, i: Inputs) -> vec3<f32> {
    if sel == 6u { return vec3(1.0); }
    return common_rgb(sel, i);
}

// Alpha a, b and d.
fn alpha_abd(sel: u32, i: Inputs) -> f32 {
    switch sel {
        case 0u: { return i.combined.a; }
        case 1u, 2u: { return i.texel.a; }
        case 3u: { return cc.prim.a; }
        case 4u: { return i.shade.a; }
        case 5u: { return cc.env.a; }
        case 6u: { return 1.0; }
        default: { return 0.0; }
    }
}

fn alpha_c(sel: u32, i: Inputs) -> f32 {
    switch sel {
        case 1u, 2u: { return i.texel.a; }
        case 3u: { return cc.prim.a; }
        case 4u: { return i.shade.a; }
        case 5u: { return cc.env.a; }
        case 6u: { return cc.extra.x; }
        default: { return 0.0; }
    }
}

fn cycle(c: vec4<u32>, a: vec4<u32>, i: Inputs) -> vec4<f32> {
    let rgb = (color_a(c.x, i) - color_b(c.y, i)) * color_c(c.z, i) + color_d(c.w, i);
    let alpha = (alpha_abd(a.x, i) - alpha_abd(a.y, i)) * alpha_c(a.z, i) + alpha_abd(a.w, i);
    return clamp(vec4(rgb, alpha), vec4(0.0), vec4(1.0));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var i: Inputs;
    i.texel = vec4(1.0);
    if (cc.flags & FLAG_TEXTURED) != 0u {
        i.texel = textureSample(tex, tex_sampler, in.uv);
    }

    // Shade: vertex colour, or simple directional + ambient lighting.
    i.shade = vec4(1.0);
#ifdef VERTEX_COLORS
    i.shade = in.color;
#endif
    if (cc.flags & FLAG_LIT) != 0u {
        let l = normalize(vec3(-0.3, 0.8, 0.5));
        let d = max(dot(normalize(in.world_normal), l), 0.0);
        i.shade = vec4(vec3(clamp(0.55 + 0.6 * d, 0.0, 1.0)), 1.0);
    }
    if (cc.flags & FLAG_NO_SHADE) != 0u {
        i.shade = vec4(1.0);
    }

    var out = i.texel;
    if (cc.flags & FLAG_NO_COMBINER) == 0u {
        i.combined = vec4(0.0);
        i.combined = cycle(cc.color0, cc.alpha0, i);
        out = i.combined;
        if (cc.flags & FLAG_TWO_CYCLE) != 0u {
            out = cycle(cc.color1, cc.alpha1, i);
        }
    }
    if out.a < 0.5 {
        discard;
    }
    return vec4(out.rgb, 1.0);
}
