//! Original shop_goods drop and player pickup/pickup_jump presentation.
//! Inventory and numeric layers commit first; this state contains no items
//! that must be recovered on save, reload or scene departure.
use super::*;
use crate::character::CharacterRig;
use crate::skeleton::{Animation, Skeleton};

const MODEL_SCALE: f32 = 0.01 / 40.0;

struct Drop {
    from: Vec3,
    current: Vec3,
    velocity: f32,
    counter: f32,
    scale: f32,
    bounced: bool,
    sounded: bool,
    remainder: f32,
}

struct Active {
    piece: loose::Piece,
    at: Vec3,
    ticks: f32,
    yaw: f32,
    offset: Vec3,
    drop: Option<Drop>,
}

#[derive(Default)]
pub(super) struct Presentation {
    active: Option<Active>,
    clips: [Option<Animation>; 2],
}

impl Presentation {
    pub(super) fn reset(&mut self) {
        self.active = None;
    }
    pub(super) fn busy(&self) -> bool {
        self.active.is_some()
    }
    pub(super) fn load(&mut self, disc: Option<&GameDisc>) {
        self.clips = ["cKF_ba_r_ply_1_pickup1", "cKF_ba_r_ply_1_clear_table1"].map(|name| {
            let disc = disc?;
            let n = disc
                .catalog
                .get("cKF_bs_r_boy_1")
                .and_then(|s| Skeleton::read(&disc.rom, s.offset))?
                .joints
                .len();
            disc.catalog
                .get(name)
                .and_then(|s| Animation::read(&disc.rom, s.offset, n))
        });
        if disc.is_some() && self.clips.iter().any(Option::is_none) {
            warn!("loose room pickup clips unavailable");
        }
    }
    pub(super) fn drop(&mut self, piece: loose::Piece, at: Vec3, player: Vec3) {
        // Shop_Goods_Actor_player_drop_entry: feet +50 source units.
        let from = player - Vec3::Y * crate::disc_town::PLAYER_HALF_HEIGHT + Vec3::Y * 1.25;
        let velocity = if (at - from).with_y(0.0).length_squared() > 0.0 {
            ((at.y - from.y) * 40.0 + 243.6) / 28.0
        } else {
            0.0
        };
        self.active = Some(Active {
            piece,
            at,
            ticks: 0.0,
            yaw: 0.0,
            offset: Vec3::ZERO,
            drop: Some(Drop {
                from,
                current: from,
                velocity,
                counter: 1.0,
                scale: 0.0,
                bounced: false,
                sounded: false,
                remainder: 0.0,
            }),
        });
    }
    pub(super) fn pickup(&mut self, piece: loose::Piece, at: Vec3, yaw: f32) {
        if self.clips[piece.layer].is_none() {
            return;
        }
        self.active = Some(Active {
            piece,
            at,
            ticks: 0.0,
            yaw,
            offset: Vec3::ZERO,
            drop: None,
        });
        if piece.layer == 1 {
            crate::sound::se_at(0x430, at); // NA_SE_DAI_UE_KAKUNOU
        }
    }
    pub(super) fn animation(&self) -> Option<(&Animation, f32, u8)> {
        let a = self.active.as_ref().filter(|a| a.drop.is_none())?;
        let clip = self.clips[a.piece.layer].as_ref()?;
        Some((
            clip,
            (1.0 + a.ticks * 0.5).min(clip.frames as f32),
            9 + a.piece.layer as u8,
        ))
    }
}

#[allow(clippy::too_many_arguments)] // One transient item owns player pose and input until finished.
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
        room.loose_presentation.reset();
        return;
    }
    if !matches!(*focus, InputFocus::World | InputFocus::Furniture) {
        return;
    }
    let presentation = &mut room.loose_presentation;
    let Some(a) = presentation.active.as_mut() else {
        return;
    };
    *focus = InputFocus::Furniture;
    if let Ok((_, _, mut loco)) = player.single_mut() {
        *loco = crate::locomotion::Locomotion::default();
    }
    // Submenu close gates the world presentation, as for the furniture leaf.
    if a.drop.is_some() && pockets.as_ref().is_some_and(|p| !p.world_ready()) {
        return;
    }
    let delta = time.delta_secs() * 60.0;
    let old = a.ticks;
    a.ticks += delta;
    let done = if let Some(drop) = a.drop.as_mut() {
        // Retail's 30Hz move (dt_num_60fps_frames=2). Fixed source-sized steps
        // keep the semi-implicit bounce independent of the display frame rate.
        drop.remainder += delta;
        let mut done = false;
        while drop.remainder >= 2.0 && !done {
            drop.remainder -= 2.0;
            let next = drop.counter + 2.0;
            let percent = (drop.counter / 30.0).min(1.0);
            drop.velocity -= 1.2;
            drop.current.y += drop.velocity / 40.0;
            drop.current.x = drop.from.x + percent * (a.at.x - drop.from.x);
            drop.current.z = drop.from.z + percent * (a.at.z - drop.from.z);
            if !drop.bounced {
                drop.scale = 0.1 + (1.0 - (1.0 - drop.counter / 28.0).powi(2)) * 0.9;
            }
            if !drop.sounded && drop.counter <= 27.0 && next > 27.0 {
                crate::sound::se_at(
                    if (0x2800..=0x2804).contains(&a.piece.item) {
                        0x127
                    } else if (0x2100..=0x2103).contains(&a.piece.item) {
                        0x444
                    } else {
                        0x128
                    },
                    drop.current,
                );
                drop.sounded = true;
            }
            if drop.current.y <= a.at.y {
                if drop.bounced {
                    done = true;
                } else {
                    drop.bounced = true;
                    drop.scale = 1.0;
                    drop.current = a.at;
                    drop.velocity = (-drop.velocity * 0.25).min(3.0);
                }
            } else {
                drop.counter = next;
            }
        }
        done
    } else {
        if a.piece.layer == 0 {
            if old < 18.0 && a.ticks >= 18.0 {
                crate::sound::se(crate::sound::se_id::ITEM_GET);
            }
            if old < 38.0 && a.ticks >= 38.0 {
                crate::sound::se(crate::sound::se_id::POCKET_RUSTLE);
            }
        } else if let Ok((transform, mut facing, _)) = player.single_mut() {
            let gap = (a.at - transform.translation).with_y(0.0);
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
        presentation.clips[a.piece.layer]
            .as_ref()
            .is_none_or(|clip| {
                a.ticks
                    >= ((clip.frames as f32 - 1.0) * 2.0).max(if a.piece.layer == 0 {
                        40.0
                    } else {
                        58.0
                    })
            })
    };
    if done {
        presentation.reset();
    }
}

#[derive(Component)]
pub(super) struct PickupVisual {
    item: u16,
}

#[allow(clippy::too_many_arguments)] // Reuse original goods meshes and the rendered player hand.
pub(super) fn render(
    mut commands: Commands,
    mut room: ResMut<PlayerRoom>,
    disc: Option<Res<GameDisc>>,
    pieces: Query<(Entity, &loose::Piece, &Transform)>,
    ghosts: Query<(Entity, &PickupVisual)>,
    rigs: Query<(Entity, &CharacterRig, &ChildOf)>,
    players: Query<&Transform, With<Player>>,
    transforms: Query<&Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<crate::n64_material::N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, piece, original) in &pieces {
        let mut transform = *original;
        transform.scale = Vec3::splat(MODEL_SCALE);
        transform.translation.y = loose::position(&room, *piece).y;
        if let Some(drop) = room
            .loose_presentation
            .active
            .as_ref()
            .filter(|a| {
                a.piece.item == piece.item
                    && a.piece.layer == piece.layer
                    && a.piece.x == piece.x
                    && a.piece.z == piece.z
            })
            .and_then(|a| a.drop.as_ref())
        {
            transform.translation = drop.current;
            transform.scale *= drop.scale;
        }
        commands.entity(entity).insert(transform);
    }
    let Some(a) = room
        .loose_presentation
        .active
        .as_mut()
        .filter(|a| a.drop.is_none())
    else {
        for (entity, _) in &ghosts {
            commands.entity(entity).despawn();
        }
        return;
    };
    let hand = rigs
        .iter()
        .find_map(|(root, rig, parent)| {
            rig.left_hand_position(
                transforms.get(root).ok()?,
                players.get(parent.parent()).ok()?,
                &transforms,
            )
        })
        .unwrap_or(a.at + Vec3::Y * 0.5);
    let frame = 1.0 + a.ticks * 0.5;
    let (position, scale) = if a.piece.layer == 0 {
        if a.ticks < 20.0 {
            a.offset = a.at - hand;
        }
        let scale = (1.0 - (a.ticks - 20.0) / 20.0).clamp(0.0, 1.0);
        (
            if a.ticks < 20.0 {
                a.at
            } else {
                hand + a.offset * scale
            },
            scale,
        )
    } else {
        if frame <= 11.0 {
            a.offset = a.at - hand;
        }
        let weight = (1.0 - (frame - 11.0) * 0.2).clamp(0.0, 1.0);
        let scale = if frame <= 11.0 {
            1.0
        } else if frame <= 16.0 {
            1.0 - (frame - 11.0) * 0.05
        } else {
            (0.75 - (frame - 16.0) * 0.75 / 14.0).max(0.0)
        };
        (
            if frame <= 11.0 {
                a.at
            } else {
                hand + a.offset * weight
            },
            scale,
        )
    };
    let transform = Transform::from_translation(position)
        .with_rotation(Quat::from_rotation_y(a.yaw))
        .with_scale(Vec3::splat(MODEL_SCALE * scale));
    let mut existing = None;
    for (entity, visual) in &ghosts {
        if visual.item == a.piece.item && existing.is_none() {
            existing = Some(entity);
        } else {
            commands.entity(entity).despawn();
        }
    }
    if let Some(entity) = existing {
        commands.entity(entity).insert(transform);
    } else if let Some(disc) = disc {
        if let Some(entity) = crate::shop_display::spawn_goods(
            &mut commands,
            &disc,
            a.piece.item,
            (a.piece.x, a.piece.z),
            corner(),
            a.at.y,
            (&mut meshes, &mut materials, &mut images),
        ) {
            commands
                .entity(entity)
                .insert((PickupVisual { item: a.piece.item }, transform));
        }
    }
}
