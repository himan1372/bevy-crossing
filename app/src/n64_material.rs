//! Material that evaluates the game's N64 colour combiner per pixel.
//!
//! The RDP combiner computes `(a - b) * c + d` for colour and alpha, once
//! (1-cycle) or twice (2-cycle, the second cycle can read the first's
//! result as `COMBINED`). [`decode_combiner`] unpacks a `G_SETCOMBINE`
//! command into mux selectors; `n64_combiner.wgsl` evaluates them with the
//! texture, shade (vertex colour or simple lighting), primitive and
//! environment colours.
//!
//! Not yet modelled: TEXEL1 (second tile; uses TEXEL0), noise, LOD
//! fraction, chroma key and the full blender. Cutouts use a 0.5 cutoff;
//! explicitly opaque panels ignore combiner alpha, as emu64 does.

use bevy::asset::{embedded_asset, RenderAssetUsages};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
    TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;

use crate::disc_gfx::{Part, PartKey, TexKey};

/// N64 texture coordinates are s10.5 fixed point.
pub const ST_ONE: f32 = 32.0;
const SHADER: &str = "embedded://bevy_crossing/n64_combiner.wgsl";

/// Flag bits in [`CombinerUniform::flags`].
pub const FLAG_TWO_CYCLE: u32 = 1;
pub const FLAG_LIT: u32 = 2;
pub const FLAG_TEXTURED: u32 = 4;
/// No combiner was set by the display lists: show the texture as is.
pub const FLAG_NO_COMBINER: u32 = 8;
/// Debug: treat shade as white.
pub const FLAG_NO_SHADE: u32 = 16;
/// Alpha-blended: keep translucent pixels instead of a 0.5 cutoff.
pub const FLAG_BLEND: u32 = 32;
/// The part is fogged (see `PartKey::fog`).
pub const FLAG_FOG: u32 = 64;
/// Explicitly opaque draw pass: retain pixels even with zero combiner alpha.
pub const FLAG_OPAQUE: u32 = 128;
/// Coplanar building ground light, drawn as a translucent decal.
pub const FLAG_GROUND_DECAL: u32 = 256;
/// Mailbox shell/lid have separate opposing inside and outside faces.
pub const FLAG_CULL_BACK: u32 = 512;
/// Submenu portrait/cursor use mSM_change_view's fixed lights, not town time.
pub(crate) const FLAG_MENU_LIGHTING: u32 = 1024;

/// Mux selectors for one cycle: colour a b c d, alpha a b c d.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CycleMux {
    pub color: [u32; 4],
    pub alpha: [u32; 4],
}

/// Decodes `G_SETCOMBINE` (w0 low 24 bits, w1) into both cycles' muxes,
/// using the RDP field layout.
pub fn decode_combiner(w0: u32, w1: u32) -> [CycleMux; 2] {
    let f = |w: u32, shift: u32, bits: u32| (w >> shift) & ((1 << bits) - 1);
    [
        CycleMux {
            color: [f(w0, 20, 4), f(w1, 28, 4), f(w0, 15, 5), f(w1, 15, 3)],
            alpha: [f(w0, 12, 3), f(w1, 12, 3), f(w0, 9, 3), f(w1, 9, 3)],
        },
        CycleMux {
            color: [f(w0, 5, 4), f(w1, 24, 4), f(w0, 0, 5), f(w1, 6, 3)],
            alpha: [f(w1, 21, 3), f(w1, 3, 3), f(w1, 18, 3), f(w1, 0, 3)],
        },
    ]
}

/// Whether cycle 2 reads cycle 1's result (`COMBINED` is mux 0 for
/// colour a/b/c/d and alpha a/b/d). Lists often leave the cycle type to
/// game code, so this is used as a hint that two cycles are meant.
fn reads_combined(c: &CycleMux) -> bool {
    c.color.contains(&0) || c.alpha[0] == 0 || c.alpha[1] == 0 || c.alpha[3] == 0
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct CombinerUniform {
    pub color0: UVec4,
    pub alpha0: UVec4,
    pub color1: UVec4,
    pub alpha1: UVec4,
    pub prim: Vec4,
    pub env: Vec4,
    /// x = primitive LOD fraction, yz = animated texture UV offset.
    pub extra: Vec4,
    pub flags: u32,
}

impl CombinerUniform {
    pub fn from_key(key: &PartKey, no_shade: bool, blend: bool) -> Self {
        let [c0, c1] = decode_combiner(key.combine.0, key.combine.1);
        let rgba = |c: [u8; 4]| Vec4::from(c.map(|v| v as f32 / 255.0));
        let mut flags = 0;
        if key.two_cycle || reads_combined(&c1) {
            flags |= FLAG_TWO_CYCLE;
        }
        if key.lit {
            flags |= FLAG_LIT;
        }
        if key.tex.is_some() {
            flags |= FLAG_TEXTURED;
        }
        if key.combine == (0, 0) {
            flags |= FLAG_NO_COMBINER;
        }
        if no_shade {
            flags |= FLAG_NO_SHADE;
        }
        if blend {
            flags |= FLAG_BLEND;
        }
        if key.fog {
            flags |= FLAG_FOG;
        }
        Self {
            color0: UVec4::from(c0.color),
            alpha0: UVec4::from(c0.alpha),
            color1: UVec4::from(c1.color),
            alpha1: UVec4::from(c1.alpha),
            prim: rgba(key.prim),
            env: rgba(key.env),
            extra: Vec4::new(key.prim_lod as f32 / 255.0, 0.0, 0.0, 0.0),
            flags,
        }
    }
}

/// Scene lighting and fog (`m_kankyo.c`), shared by every material and
/// updated by `environment`. Colours 0..1; directions point at the light;
/// `fog` = (near, far) in the N64 depth range 0..1000, enabled, unused.
#[derive(Clone, Copy, Debug, ShaderType)]
pub struct EnvUniform {
    pub ambient: Vec4,
    pub sun_dir: Vec4,
    pub sun_color: Vec4,
    pub moon_dir: Vec4,
    pub moon_color: Vec4,
    pub fog_color: Vec4,
    pub fog: Vec4,
}

impl Default for EnvUniform {
    /// Plain daylight, no fog (model viewer, or before the disc's table loads).
    fn default() -> Self {
        Self {
            ambient: Vec4::splat(0.55),
            sun_dir: Vec3::new(-0.3, 0.8, 0.5).normalize().extend(0.0),
            sun_color: Vec4::splat(0.6),
            moon_dir: Vec4::Y,
            moon_color: Vec4::ZERO,
            fog_color: Vec4::ZERO,
            fog: Vec4::new(1000.0, 1000.0, 0.0, 0.0),
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(N64MaterialKey)]
pub struct N64Material {
    #[uniform(0)]
    pub combiner: CombinerUniform,
    #[uniform(3)]
    pub env: EnvUniform,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
    /// Translucent (`*_modelT`, XLU) geometry is alpha-blended. Other
    /// parts retain the existing cutout policy unless marked explicitly opaque.
    pub blend: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct N64MaterialKey {
    ground_decal: bool,
    cull_back: bool,
}

impl From<&N64Material> for N64MaterialKey {
    fn from(material: &N64Material) -> Self {
        Self {
            ground_decal: material.combiner.flags & FLAG_GROUND_DECAL != 0,
            cull_back: material.combiner.flags & FLAG_CULL_BACK != 0,
        }
    }
}

impl Material for N64Material {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    /// Keep the existing double-sided policy except for explicitly marked
    /// source-backed culling, such as the mailbox's opposing shell faces.
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = key
            .bind_group_data
            .cull_back
            .then_some(bevy::render::render_resource::Face::Back);
        if key.bind_group_data.ground_decal {
            if let Some(depth) = descriptor.depth_stencil.as_mut() {
                // Bevy uses reverse Z (GreaterEqual): positive bias brings
                // the coplanar decal toward the camera. Keep depth testing
                // so buildings/characters still occlude the light patch.
                depth.bias.constant = 2;
                depth.bias.slope_scale = 1.0;
                depth.depth_write_enabled = Some(false);
            }
        }
        Ok(())
    }

    fn alpha_mode(&self) -> AlphaMode {
        if self.blend {
            AlphaMode::Blend
        } else if self.combiner.flags & FLAG_OPAQUE != 0 {
            AlphaMode::Opaque
        } else {
            AlphaMode::Mask(0.5)
        }
    }
}

/// Unlit parts carry vertex colours (the combiner's SHADE input); lit
/// parts carry the vertex normals stored in the same bytes.
pub fn part_mesh(part: &Part, scale: f32) -> Mesh {
    let (w, h) = part
        .key
        .tex
        .map_or((1.0, 1.0), |k| (k.width as f32, k.height as f32));
    let v = &part.vertices;
    let positions: Vec<[f32; 3]> = v.iter().map(|v| v.pos.map(|c| c * scale)).collect();
    let uvs: Vec<[f32; 2]> = v
        .iter()
        .map(|v| [v.st[0] / ST_ONE / w, v.st[1] / ST_ONE / h])
        .collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    if part.key.lit {
        let normals: Vec<[f32; 3]> = v.iter().map(|v| v.normal).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    } else {
        let colors: Vec<[f32; 4]> = v
            .iter()
            .map(|v| v.color.map(|c| c as f32 / 255.0))
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.compute_flat_normals();
    }
    mesh
}

fn address_mode(gx_wrap: u8) -> ImageAddressMode {
    match gx_wrap {
        1 => ImageAddressMode::Repeat,
        2 => ImageAddressMode::MirrorRepeat,
        _ => ImageAddressMode::ClampToEdge,
    }
}

pub fn texture_image(rgba: Vec<u8>, key: &TexKey) -> Image {
    let size = Extent3d {
        width: key.width as u32,
        height: key.height as u32,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        rgba,
        // Raw texel values: the combiner works on them as the GameCube
        // does, alongside the raw vertex/prim/env colours, and the shader
        // converts the result to linear once.
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: address_mode(key.wrap_s),
        address_mode_v: address_mode(key.wrap_t),
        ..ImageSamplerDescriptor::linear()
    });
    image
}

pub struct N64MaterialPlugin;

impl Plugin for N64MaterialPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "n64_combiner.wgsl");
        app.add_plugins(MaterialPlugin::<N64Material>::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_most_common_combiner() {
        // The disc's most-used combiner: TEXEL0 * SHADE, then PRIM * COMBINED;
        // alpha TEXEL0, then COMBINED.
        let [c0, c1] = decode_combiner(0x12_7E60, 0xFFFF_F3F8);
        assert_eq!(c0.color, [1, 15, 4, 7]);
        assert_eq!(c0.alpha, [7, 7, 7, 1]);
        assert_eq!(c1.color, [3, 15, 0, 7]);
        assert_eq!(c1.alpha, [7, 7, 7, 0]);
        assert!(reads_combined(&c1));
    }

    #[test]
    fn decodes_model_setup_combiner() {
        // m_rcp.c setup: G_CC_MODULATEIDECALA, G_CC_MODULATEIA_PRIM2.
        let [c0, c1] = decode_combiner(0x12_7E03, 0xFF0F_F3FF);
        assert_eq!(c0.color, [1, 15, 4, 7]);
        assert_eq!(c0.alpha, [7, 7, 7, 1]);
        assert_eq!(c1.color, [0, 15, 3, 7]);
        assert_eq!(c1.alpha, [0, 7, 3, 7]);
    }

    #[test]
    fn single_cycle_shade_only() {
        // G_CC_SHADE in both cycles: (0 - 0) * 0 + SHADE.
        let w1 = 15 << 28
            | 15 << 24
            | 7 << 21
            | 7 << 18
            | 4 << 15
            | 7 << 12
            | 4 << 9
            | 4 << 6
            | 7 << 3
            | 4;
        let [c0, c1] = decode_combiner(0xFF_FFFF, w1);
        assert_eq!(c0.color[3], 4);
        assert_eq!(c0.alpha[3], 4);
        assert_eq!(c0.color[3], c1.color[3]);
        assert!(!reads_combined(&c1));
    }
}
