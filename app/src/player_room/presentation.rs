//! ac_my_room reservation/birth/bye and m_player_main_pickup_furniture.
//! Numeric transactions finish first; this state owns only temporary visuals.
use super::*;
use crate::character::CharacterRig;
use crate::skeleton::{Animation, Skeleton};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Birth,
    Pickup,
}

struct Transition {
    kind: Kind,
    slot: usize,
    item: u16,
    at: Vec3,
    from: Vec3,
    ticks: f32,
    birth: ftr::FurnitureBirth,
    leaf_scale: f32,
}

#[derive(Default)]
pub(super) struct Presentation {
    active: Option<Transition>,
    pickup_clip: Option<Animation>,
    hand_offset: Vec3,
}

impl Presentation {
    pub(super) fn reset(&mut self) {
        self.active = None;
        self.hand_offset = Vec3::ZERO;
    }
    pub(super) fn load(&mut self, disc: Option<&GameDisc>) {
        self.pickup_clip = disc.and_then(|disc| {
            let n = disc
                .catalog
                .get("cKF_bs_r_boy_1")
                .and_then(|s| Skeleton::read(&disc.rom, s.offset))?
                .joints
                .len();
            disc.catalog
                .get("cKF_ba_r_ply_1_pickup1")
                .and_then(|s| Animation::read(&disc.rom, s.offset, n))
        });
        if disc.is_some() && self.pickup_clip.is_none() {
            warn!("furniture pickup clip unavailable");
        }
    }
    pub(super) fn busy(&self) -> bool {
        self.active.is_some()
    }
    pub(super) fn animating(&self, slot: usize) -> bool {
        self.active.as_ref().is_some_and(|a| a.slot == slot)
    }
    pub(super) fn picking_up(&self, slot: usize) -> bool {
        self.active
            .as_ref()
            .is_some_and(|a| a.kind == Kind::Pickup && a.slot == slot)
    }
    pub(super) fn birth(&mut self, slot: usize, item: u16, at: Vec3, player: Vec3, facing: Vec3) {
        // eKagu_Happa_ct: 20 units above the feet and 10 units forward.
        let from =
            player - Vec3::Y * crate::disc_town::PLAYER_HALF_HEIGHT + Vec3::Y * 0.5 + facing * 0.25;
        self.active = Some(Transition {
            kind: Kind::Birth,
            slot,
            item,
            at,
            from,
            ticks: 0.0,
            birth: default(),
            leaf_scale: 0.0,
        });
    }
    pub(super) fn pickup(&mut self, slot: usize, item: u16, at: Vec3) {
        self.hand_offset = Vec3::ZERO;
        self.active = Some(Transition {
            kind: Kind::Pickup,
            slot,
            item,
            at,
            from: at,
            ticks: 0.0,
            birth: default(),
            leaf_scale: 1.0,
        });
        crate::sound::se_at(0x58, at); // NA_SE_CLEAN_UP_FTR
    }
    pub(super) fn animation(&self) -> Option<(&Animation, f32, u8)> {
        let active = self.active.as_ref().filter(|a| a.kind == Kind::Pickup)?;
        let clip = self.pickup_clip.as_ref()?;
        Some((clip, (1.0 + active.ticks * 0.5).min(clip.frames as f32), 5))
    }
}

pub(super) fn actor_center(room: &PlayerRoom, a: &ftr::FurnitureActor) -> Vec3 {
    let cells = a.shape_type.unit_offsets();
    let count = a.shape_type.unit_count();
    let offset = cells
        .iter()
        .take(count)
        .fold(Vec2::ZERO, |p, &(x, z)| p + Vec2::new(x as f32, z as f32))
        / count as f32;
    let y = if a.layer == 1 {
        room.runtime.occupancy[0][a.cell_z as usize][a.cell_x as usize]
            .and_then(|id| room.runtime.actors[id as usize].as_ref())
            .and_then(|parent| room.profiles.get(&parent.name))
            .map_or(0.0, |p| p.height / 40.0)
    } else {
        0.0
    };
    corner()
        + Vec3::new(
            a.cell_x as f32 + 0.5 + offset.x,
            y,
            a.cell_z as f32 + 0.5 + offset.y,
        )
}

pub(super) fn advance(
    mut room: ResMut<PlayerRoom>,
    current: Res<CurrentRoom>,
    state: Res<State<HouseState>>,
    time: Res<Time>,
    mut focus: ResMut<InputFocus>,
    pockets: Option<Res<crate::pockets_disc::PocketAssets>>,
    mut player: Query<(&Transform, &mut Facing, &mut crate::locomotion::Locomotion), With<Player>>,
) {
    if !crate::house::is_player_room(&current, &state) {
        room.presentation.reset();
        return;
    }
    if !matches!(*focus, InputFocus::World | InputFocus::Furniture) {
        return;
    }
    let pickup_end = room
        .presentation
        .pickup_clip
        .as_ref()
        .map_or(40.0, |a| (a.frames as f32 - 1.0) * 2.0);
    let Some(active) = room.presentation.active.as_mut() else {
        return;
    };
    *focus = InputFocus::Furniture;
    if active.kind == Kind::Birth && pockets.as_ref().is_some_and(|p| !p.world_ready()) {
        return;
    }
    let delta = time.delta_secs() * 60.0;
    let old = active.ticks;
    active.ticks += delta;
    if let Ok((transform, mut facing, mut loco)) = player.single_mut() {
        *loco = crate::locomotion::Locomotion::default();
        if active.kind == Kind::Pickup {
            let gap = (active.at - transform.translation).with_y(0.0);
            if gap.length_squared() > 0.0001 {
                let yaw = facing.dir.x.atan2(facing.dir.z);
                let target = gap.x.atan2(gap.z);
                let error = (target - yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                    - std::f32::consts::PI;
                let step = (error * (1.0 - 0.5f32.sqrt().powf(delta))).clamp(
                    -13.73291f32.to_radians() * delta,
                    13.73291f32.to_radians() * delta,
                );
                facing.dir = Vec3::new((yaw + step).sin(), 0.0, (yaw + step).cos());
            }
        }
    }
    let done = if active.kind == Kind::Birth {
        // Existing native reservation duration, followed by aMR_FtrBirth.
        let wait = ftr::RSV_FRAME_COUNT as f32;
        if old < wait && active.ticks >= wait {
            crate::sound::se_at(0x57, active.at);
        }
        active
            .birth
            .advance((active.ticks - wait).max(0.0) - (old - wait).max(0.0));
        // add_calc(target .01, CALC_EASE(.25), max .0015, min .00005).
        let ease = 1.0 - (0.75f32).sqrt();
        let step = (1.0 - active.leaf_scale) * (1.0 - (1.0 - ease).powf(delta));
        active.leaf_scale = (active.leaf_scale + step.clamp(0.005 * delta, 0.15 * delta)).min(1.0);
        active.ticks >= wait && active.birth.finished
    } else {
        if old < 18.0 && active.ticks >= 18.0 {
            crate::sound::se(crate::sound::se_id::ITEM_GET);
        }
        if old < 38.0 && active.ticks >= 38.0 {
            crate::sound::se(crate::sound::se_id::POCKET_RUSTLE);
        }
        active.ticks >= pickup_end.max(38.0)
    };
    if done {
        room.presentation.reset();
    }
}

/// Retained geometry after its inventory/foreground transaction has finished.
#[derive(Component)]
pub(super) struct RetiringPiece {
    pub(super) base: Transform,
    pub(super) long: bool,
}
#[derive(Component)]
pub(super) struct Leaf {
    icon: usize,
}
type Model = Vec<(Handle<Mesh>, Handle<crate::n64_material::N64Material>)>;
#[derive(Default)]
pub(super) struct LeafCache(HashMap<usize, Model>);

fn model(
    disc: &GameDisc,
    icon: usize,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<crate::n64_material::N64Material>,
    images: &mut Assets<Image>,
) -> Model {
    let Some(table) = disc.catalog.get("aMR_icon_display_data") else {
        return Vec::new();
    };
    let lists = (0..2)
        .filter_map(|i| {
            match disc
                .rom
                .pointer_at(table.source, table.offset + (icon * 8 + i * 4) as u32)
            {
                Some(Target::File(RomSource::Rel, p)) => Some(p),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    if lists.len() != 2 {
        return Vec::new();
    }
    crate::disc_town::build_model(disc, &lists, None, meshes, materials, images)
}

fn scaled(mut base: Transform, long: bool, scale: Vec3) -> Transform {
    // aMR_DrawFurniture scales TypeB about its half-unit model centre.
    if long {
        base.translation += base.rotation * Vec3::new(0.5 * (1.0 - scale.x), 0.0, 0.0);
    }
    base.scale *= scale;
    base
}

#[allow(clippy::too_many_arguments)] // Render the one transaction with shared disc/model caches.
pub(super) fn render(
    mut commands: Commands,
    mut room: ResMut<PlayerRoom>,
    disc: Option<Res<GameDisc>>,
    pieces: Query<(Entity, &RoomPiece, &Transform)>,
    ghosts: Query<(Entity, &RetiringPiece)>,
    leaves: Query<(Entity, &Leaf)>,
    rigs: Query<(Entity, &CharacterRig, &ChildOf)>,
    players: Query<&Transform, With<Player>>,
    cameras: Query<&Transform, With<crate::player::WorldCamera>>,
    transforms: Query<&Transform>,
    mut cache: Local<LeafCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<crate::n64_material::N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, piece, transform) in &pieces {
        let Some(index) = crate::furniture::furniture_index(piece.item) else {
            continue;
        };
        let Some(p) = room.profiles.get(&(index as u16)) else {
            continue;
        };
        let mut base = *transform;
        base.scale = Vec3::splat(p.scale / 40.0);
        let factor = room
            .presentation
            .active
            .as_ref()
            .filter(|a| a.kind == Kind::Birth && a.slot == piece.slot)
            .map_or(Vec3::ONE, |a| {
                if a.ticks < ftr::RSV_FRAME_COUNT as f32 {
                    Vec3::ZERO
                } else {
                    Vec3::from_array(a.birth.scale)
                }
            });
        commands
            .entity(entity)
            .insert(scaled(base, p.shape.unit_count() == 2, factor));
    }
    for (entity, ghost) in &ghosts {
        let scale = room
            .presentation
            .active
            .as_ref()
            .filter(|a| a.kind == Kind::Pickup)
            .map_or(0.0, |a| ftr::pickup_visual_scale(a.ticks));
        if scale == 0.0 {
            commands.entity(entity).despawn();
        } else {
            commands
                .entity(entity)
                .insert(scaled(ghost.base, ghost.long, Vec3::splat(scale)));
        }
    }
    let Some(active) = room.presentation.active.as_ref() else {
        for (entity, _) in &leaves {
            commands.entity(entity).despawn();
        }
        return;
    };
    let (kind, ticks, item, at, from, leaf_scale) = (
        active.kind,
        active.ticks,
        active.item,
        active.at,
        active.from,
        active.leaf_scale,
    );
    let camera = cameras.single().ok().map_or(Quat::IDENTITY, |c| c.rotation);
    let leaf = if kind == Kind::Birth && (1.0..37.0).contains(&ticks) {
        // eKagu_Happa_mv: reverse countdown interpolation and a parabolic hop.
        let timer = 37.0 - ticks;
        let xy = ((timer - 8.0) / 28.0).clamp(0.0, 1.0);
        let z = ((timer - 8.0) / 20.0).clamp(0.0, 1.0);
        let mut pos = at.lerp(from, xy);
        pos.z = at.z + (from.z - at.z) * z;
        if timer > 10.0 {
            pos.y += ((timer - 18.0).powi(2) * (-10.0 / 98.0) + 20.0) / 40.0;
        }
        let roll = ((ticks - 1.0) * 0x924 as f32 * std::f32::consts::TAU / 65536.0).sin()
            * 5120.0
            * std::f32::consts::TAU
            / 65536.0;
        Some(
            Transform::from_translation(pos)
                .with_rotation(Quat::from_rotation_z(roll) * camera)
                .with_scale(Vec3::splat(leaf_scale)),
        )
    } else if kind == Kind::Pickup && ftr::pickup_visual_scale(ticks) == 0.0 {
        let hand = rigs
            .iter()
            .find_map(|(entity, rig, parent)| {
                let player = players.get(parent.parent()).ok()?;
                rig.left_hand_position(transforms.get(entity).ok()?, player, &transforms)
            })
            .unwrap_or(at + Vec3::Y * 0.5);
        let frame = 1.0 + ticks * 0.5;
        if frame <= 10.0 {
            room.presentation.hand_offset = at - hand;
        }
        let scale = (1.0 - (frame - 10.0) / 10.0).clamp(0.0, 1.0);
        Some(
            Transform::from_translation(hand + room.presentation.hand_offset * scale)
                .with_scale(Vec3::splat(scale)),
        )
    } else {
        None
    };
    let Some(transform) = leaf else {
        for (entity, _) in &leaves {
            commands.entity(entity).despawn();
        }
        return;
    };
    let icon = ftr::furniture_icon(item);
    let mut existing = None;
    for (entity, leaf) in &leaves {
        if leaf.icon == icon && existing.is_none() {
            existing = Some(entity);
        } else {
            commands.entity(entity).despawn();
        }
    }
    if let Some(entity) = existing {
        commands.entity(entity).insert(transform);
        return;
    }
    let Some(disc) = disc else { return };
    let parts = cache
        .0
        .entry(icon)
        .or_insert_with(|| model(&disc, icon, &mut meshes, &mut materials, &mut images));
    if parts.is_empty() {
        return;
    }
    let root = commands
        .spawn((
            Leaf { icon },
            Name::new("furniture leaf"),
            transform,
            Visibility::default(),
        ))
        .id();
    for (mesh, material) in parts.iter() {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Visibility::default(),
            ChildOf(root),
        ));
    }
}
