//! Gong one-shot cKF playback and ringside-wall camera flashes.
use super::*;
use crate::disc_gfx::{decode_texture, run_display_lists_with_prim, Part, TexKey};
use crate::house::{CurrentRoom, HouseState};
use crate::n64_material::{texture_image, CombinerUniform};
use crate::player_room::{PlayerRoom, RoomPiece};
use rustimal_logic::furniture_gong as native;
use rustimal_logic::graphics::texture::GxTexFormat;

pub(crate) fn is_gong(index: u16) -> bool {
    index == 0x4EE
}

fn flash_parts(disc: &GameDisc) -> Option<Vec<Part>> {
    let mode = disc.catalog.get("ef_takurami01_normal_render_mode")?;
    let model = disc.catalog.get("ef_takurami01_kira_modelT")?;
    let mut parts = run_display_lists_with_prim(
        &disc.rom,
        &[mode.offset, model.offset],
        [255, 255, 255, 200],
        255,
    )
    .parts;
    // Like sunshine's classic load-block lists, this asset's plain SETTILE
    // commands don't populate the interpreter's Dolphin texture metadata.
    // Reconstruct this exact IA8 16x16, mirror/mirror tile locally.
    let texture = disc.catalog.get("ef_takurami01_1")?;
    for part in &mut parts {
        part.key.tex = Some(TexKey {
            img_addr: texture.offset,
            format: GxTexFormat::IA4,
            width: 16,
            height: 16,
            wrap_s: 2,
            wrap_t: 2,
            tlut_addr: 0,
        });
    }
    (!parts.is_empty()).then_some(parts)
}

/// ef_flash's legacy gsDPLoadBlock texture is row-major N64 IA8, not
/// Dolphin/GX-tiled IA4. Keep this known asset conversion local to the effect.
fn flash_rgba(disc: &GameDisc, part: &Part) -> Option<Vec<u8>> {
    let key = part.key.tex.as_ref()?;
    let asset = disc.catalog.get("ef_takurami01_1")?;
    if key.img_addr != asset.offset
        || key.width != 16
        || key.height != 16
        || key.format != GxTexFormat::IA4
    {
        return None;
    }
    Some(
        disc.rom
            .slice(asset.source, asset.offset, 256)?
            .iter()
            .flat_map(|v| {
                let i = (v >> 4) * 17;
                [i, i, i, (v & 15) * 17]
            })
            .collect(),
    )
}

pub(super) fn available(disc: &GameDisc, profile: &Profile) -> bool {
    let Some((skeleton, Some(_))) = profile_skeleton(disc, profile) else {
        warn!("incomplete gong skeleton or animation");
        return false;
    };
    let mut count = 0;
    for joint in &skeleton.joints {
        if let Some(model) = joint.model {
            for part in run_display_lists(&disc.rom, &[model]).parts {
                count += 1;
                if part.key.tex.as_ref().is_some_and(|key| {
                    decode_texture(&disc.rom, key, profile.palette, &Segments::default()).is_none()
                }) {
                    warn!("incomplete gong body texture: {:?}", part.key.tex);
                    return false;
                }
            }
        }
    }
    count > 0 && flash_parts(disc).is_some_and(|p| p.iter().all(|p| flash_rgba(disc, p).is_some()))
}

#[derive(Component)]
pub(crate) struct GongRig {
    skeleton: Skeleton,
    animation: Animation,
    joints: Vec<Entity>,
    state: native::Gong,
    switch: Option<u8>,
    pending: bool,
    ticks: f32,
}

pub(super) fn spawn(
    commands: &mut Commands,
    disc: &GameDisc,
    root: Entity,
    profile: &Profile,
    assets: (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let Some((skeleton, Some(animation))) = profile_skeleton(disc, profile) else {
        return;
    };
    let lists = skeleton
        .joints
        .iter()
        .enumerate()
        .filter_map(|(i, j)| Some((i, j.model?, j.flags & 1 != 0)))
        .collect();
    let joints = super::rigs::spawn_skinned(
        commands,
        disc,
        root,
        "gong joint",
        &skeleton,
        &animation,
        lists,
        profile.palette,
        &Segments::default(),
        assets,
    );
    commands.entity(root).insert(GongRig {
        skeleton,
        animation,
        joints,
        state: default(),
        switch: None,
        pending: false,
        ticks: 0.0,
    });
}

#[derive(Resource, Default)]
pub(crate) struct Effects {
    state: native::Flashes,
    ticks: f32,
    sprites: Vec<Entity>,
    assets: Option<Vec<(Handle<Mesh>, Handle<N64Material>)>>,
}

impl Effects {
    pub(crate) fn light(&self) -> f32 {
        f32::from(self.state.light) / 255.0
    }

    fn clear(&mut self, commands: &mut Commands) {
        self.state.clear();
        self.ticks = 0.0;
        for sprite in self.sprites.drain(..) {
            commands.entity(sprite).despawn();
        }
    }
}

/// Source room-S coordinates to this port's room units, as sunshine uses.
fn world(pos: [f32; 3], origin: Vec3) -> Vec3 {
    Vec3::from(pos) / 40.0 - Vec3::new(3.0, 1.0, 3.0) + origin
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update(
    mut commands: Commands,
    time: Res<Time>,
    house: Res<State<HouseState>>,
    current: Res<CurrentRoom>,
    room: Res<PlayerRoom>,
    disc: Option<Res<GameDisc>>,
    mut rigs: Query<(&mut GongRig, &GlobalTransform, Option<&RoomPiece>)>,
    mut transforms: Query<&mut Transform>,
    camera: Query<&GlobalTransform, With<crate::player::WorldCamera>>,
    mut effects: ResMut<Effects>,
    mut loads: MessageReader<crate::sound::ResetAudio>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let loaded = loads.read().count() != 0;
    let inside = crate::house::is_player_room(&current, &house);
    if loaded || !inside {
        effects.clear(&mut commands);
    }
    for (mut rig, at, piece) in &mut rigs {
        let switch = piece.and_then(|p| room.switch_bit(p));
        rig.pending |= rig.switch.is_some() && switch.is_some() && switch != rig.switch;
        rig.switch = switch;
        if inside {
            rig.ticks = (rig.ticks + time.delta_secs() * 60.0).min(4.0);
        }
        while rig.ticks >= 1.0 {
            rig.ticks -= 1.0;
            let changed = std::mem::take(&mut rig.pending);
            let end = rig.animation.frames;
            if rig.state.step(changed, end)
                && inside
                && piece.is_some_and(|p| room.clock_sound_allowed(p))
            {
                crate::sound::se_at(0x174, at.translation());
                if room.ringside_wall() {
                    crate::sound::se(0x817E);
                    effects.state.start();
                }
            }
        }
        for (&joint, local) in rig.joints.iter().zip(
            rig.animation
                .pose(rig.state.pose)
                .local_transforms(&rig.skeleton),
        ) {
            if let Ok(mut transform) = transforms.get_mut(joint) {
                *transform = local;
            }
        }
    }
    if !inside {
        return;
    }
    effects.ticks = (effects.ticks + time.delta_secs() * 60.0).min(4.0);
    while effects.ticks >= 1.0 {
        effects.ticks -= 1.0;
        effects.state.step();
    }
    if effects.state.particles.is_empty() {
        for sprite in effects.sprites.drain(..) {
            commands.entity(sprite).despawn();
        }
        return;
    }
    if effects.assets.is_none() {
        let Some(disc) = disc else {
            effects.clear(&mut commands);
            return;
        };
        let Some(parts) = flash_parts(&disc) else {
            return;
        };
        let assets: Option<Vec<_>> = parts
            .iter()
            .map(|part| {
                let key = part.key.tex.as_ref()?;
                let rgba = flash_rgba(&disc, part)?;
                let image = images.add(texture_image(rgba, key));
                let material = materials.add(N64Material {
                    combiner: CombinerUniform::from_key(&part.key, false, true),
                    texture: Some(image),
                    env: default(),
                    blend: true,
                });
                Some((
                    meshes.add(crate::n64_material::part_mesh(part, 1.0)),
                    material,
                ))
            })
            .collect();
        effects.assets = assets;
    }
    let Some(assets) = effects.assets.clone() else {
        return;
    };
    let count = effects.state.particles.len();
    while effects.sprites.len() < count {
        let root = commands
            .spawn((
                Name::new("gong camera flash"),
                Transform::default(),
                Visibility::default(),
            ))
            .id();
        for (mesh, material) in &assets {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                ChildOf(root),
            ));
        }
        effects.sprites.push(root);
    }
    while effects.sprites.len() > count {
        if let Some(entity) = effects.sprites.pop() {
            commands.entity(entity).despawn();
        }
    }
    let rotation = camera.single().map_or(Quat::IDENTITY, |c| c.rotation());
    for (&entity, flash) in effects.sprites.iter().zip(&effects.state.particles) {
        let transform = Transform {
            translation: world(flash.position, current.origin()),
            rotation,
            scale: Vec3::splat(flash.scale() / 40.0),
        };
        // Newly spawned roots are deferred; insertion also updates existing roots.
        commands.entity(entity).insert(transform);
    }
}
