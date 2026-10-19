// N64 colour combiner: (a - b) * c + d, one or two cycles.
// Mux numbering follows the RDP (see n64_material.rs).

#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct Combiner {
    color0: vec4<u32>,
    alpha0: vec4<u32>,
    color1: vec4<u32>,
    alpha1: vec4<u32>,
    prim: vec4<f32>,
    env: vec4<f32>,
    extra: vec4<f32>,
    // G_TEXTURE_GEN: uv = (s', t') * xy - zw.
    texgen: vec4<f32>,
    flags: u32,
};

const FLAG_TWO_CYCLE: u32 = 1u;
const FLAG_LIT: u32 = 2u;
const FLAG_TEXTURED: u32 = 4u;
const FLAG_NO_COMBINER: u32 = 8u;
const FLAG_NO_SHADE: u32 = 16u;
const FLAG_BLEND: u32 = 32u;
const FLAG_FOG: u32 = 64u;
const FLAG_TEXGEN: u32 = 2048u;
const FLAG_OPAQUE: u32 = 128u;

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> cc: Combiner;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var tex_sampler: sampler;

struct Env {
    ambient: vec4<f32>,
    sun_dir: vec4<f32>,
    sun_color: vec4<f32>,
    moon_dir: vec4<f32>,
    moon_color: vec4<f32>,
    fog_color: vec4<f32>,
    // GX fog start and end eye depths (game units), enabled, unused
    fog: vec4<f32>,
    // Room point light as a per-object directional light; black elsewhere.
    point_dir: vec4<f32>,
    point_color: vec4<f32>,
    effect_color: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> env: Env;

// World units are cells; the game measures 40 units per cell.
const GAME_UNITS: f32 = 40.0;
// The combiner's output is a display (sRGB) value, like the GameCube's;
// the render target wants linear.
fn display_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3(2.4));
    return select(hi, lo, c <= vec3(0.04045));
}

// GX_FOG_PERSP_LIN (emu64's GXSetFog): linear in eye depth from the
// start to the end depth the CPU worked out from gSPFogPosition.
fn fog_amount(world_pos: vec3<f32>) -> f32 {
    let forward = -normalize(view.world_from_view[2].xyz);
    let depth = dot(world_pos - view.world_position, forward) * GAME_UNITS;
    let range = max(env.fog.y - env.fog.x, 1.0);
    return clamp((depth - env.fog.x) / range, 0.0, 1.0);
}

// emu64 texture_gen: the world normal (modelview here holds only model
// matrices) projected on the look-at axes guLookAtHilite stores as s8
// fractions: X = camera right, Y = camera up.
fn texgen_uv(n: vec3<f32>) -> vec2<f32> {
    // FTOFRAC8: (s8)MIN(x * 128, 127), truncating.
    let q = 128.0;
    let right = min(trunc(normalize(view.world_from_view[0].xyz) * q), vec3(127.0)) / q;
    let up = min(trunc(normalize(view.world_from_view[1].xyz) * q), vec3(127.0)) / q;
    let st = vec2(dot(right, n), dot(up, n)) * 0.5 + 0.5;
    return st * cc.texgen.xy - cc.texgen.zw;
}

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
        var uv = in.uv;
        if (cc.flags & FLAG_TEXGEN) != 0u {
            uv = texgen_uv(normalize(in.world_normal));
        }
        i.texel = textureSample(tex, tex_sampler, uv + cc.extra.yz);
    }

    // Shade: vertex colour, or the scene's ambient + sun + moon lights.
    i.shade = vec4(1.0);
#ifdef VERTEX_COLORS
    i.shade = in.color;
#endif
    if (cc.flags & FLAG_LIT) != 0u {
        let n = normalize(in.world_normal);
        let lit = env.ambient.rgb
            + env.sun_color.rgb * max(dot(n, env.sun_dir.xyz), 0.0)
            + env.moon_color.rgb * max(dot(n, env.moon_dir.xyz), 0.0)
            + env.point_color.rgb * max(dot(n, env.point_dir.xyz), 0.0)
            + env.effect_color.rgb * max(n.y, 0.0);
        i.shade = vec4(clamp(lit, vec3(0.0), vec3(1.0)), 1.0);
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
    if env.fog.z > 0.0 && (cc.flags & FLAG_FOG) != 0u {
        out = vec4(mix(out.rgb, env.fog_color.rgb, fog_amount(in.world_position.xyz)), out.a);
    }
    if (cc.flags & FLAG_BLEND) != 0u {
        if out.a < 0.01 {
            discard;
        }
        return vec4(display_to_linear(clamp(out.rgb, vec3(0.0), vec3(1.0))), out.a);
    }
    if (cc.flags & FLAG_OPAQUE) == 0u && out.a < 0.5 {
        discard;
    }
    return vec4(display_to_linear(clamp(out.rgb, vec3(0.0), vec3(1.0))), 1.0);
}
