//! Adapter for native outdoor intro cameras. The player's camera remains the sole writer.
use bevy::prelude::*;
use rustimal_logic::intro_camera::View;

/// Native actor culling width; keeps profile selection out of camera queries.
#[derive(Component)]
pub(crate) struct Occluder {
    width: f32,
    restore: Option<Visibility>,
}

impl Occluder {
    pub(crate) fn new(width: f32) -> Self {
        Self {
            width: width / 40.0,
            restore: None,
        }
    }

    /// A cached shop may change tier while the camera temporarily hides it.
    /// Update its underlying visibility rather than restoring a stale tier later.
    pub(crate) fn set_base_visibility(
        commands: &mut Commands,
        entity: Entity,
        visibility: Visibility,
    ) {
        commands.queue(move |world: &mut World| {
            let Ok(mut entity) = world.get_entity_mut(entity) else {
                return;
            };
            let culled = entity.get_mut::<Self>().is_some_and(|mut occluder| {
                if occluder.restore.is_some() {
                    occluder.restore = Some(visibility);
                    true
                } else {
                    false
                }
            });
            if !culled {
                entity.insert(visibility);
            }
        });
    }
}

#[derive(Resource, Default)]
pub(crate) struct Occlusion {
    ticks: f32,
    area: Option<(Vec3, Vec3, Vec3, Vec3)>,
}

impl Occlusion {
    pub(crate) fn update(
        &mut self,
        actors: Option<(Vec3, Vec3)>,
        view: Option<(Vec3, Vec3)>,
        frames: f32,
    ) {
        if let Some(((player, npc), (center, eye))) = actors.zip(view) {
            self.ticks = (self.ticks + frames).min(15.0);
            self.area = (self.ticks >= 15.0).then_some((center, eye, player, npc));
        } else {
            *self = Self::default();
        }
    }
}

fn cull(
    area: Res<Occlusion>,
    mut objects: Query<(&Transform, &mut Visibility, &mut Occluder)>,
    mut was_active: Local<bool>,
) {
    if area.area.is_none() && !*was_active {
        return;
    }
    *was_active = area.area.is_some();
    for (at, mut visibility, mut object) in &mut objects {
        // Camera2_CheckEnterCullingArea / Talk_SetTalking_Cull.
        let hidden = area.area.is_some_and(|(center, eye, player, npc)| {
            at.translation.z >= center.z
                && at.translation.x >= player.x.min(npc.x).min(eye.x) - object.width
                && at.translation.x <= player.x.max(npc.x).max(eye.x) + object.width
        });
        if hidden {
            if object.restore.is_none() && *visibility != Visibility::Hidden {
                object.restore = Some(*visibility);
                *visibility = Visibility::Hidden;
            }
        } else if let Some(original) = object.restore.take() {
            *visibility = original;
        }
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<Occlusion>().add_systems(
        Update,
        cull.after(crate::player::camera_follow)
            .in_set(crate::states::GameplaySet),
    );
}

#[derive(Default)]
pub(crate) struct Camera {
    current: Option<View>,
    previous: Option<View>,
    ticks: f32,
}

impl Camera {
    pub(crate) fn view(
        &mut self,
        request: Option<View>,
        held: bool,
        normal: Vec3,
        normal_target: Vec3,
        frames: f32,
        active: bool,
        outdoor: bool,
    ) -> Option<(Vec3, Vec3)> {
        if !outdoor {
            *self = Self::default();
            return None;
        }
        if !active && request.is_none() && self.current.is_none() {
            return None;
        }
        let goal = request.unwrap_or_else(|| View::normal((normal_target * 40.0).to_array()));
        let current = self
            .current
            .get_or_insert_with(|| View::normal((normal * 40.0).to_array()));
        let gap = Vec3::from_array(goal.center) - Vec3::from_array(current.center);
        // Scene changes and QuickLoad never sweep across the intervening world.
        if held || gap.length() > 620.0 {
            *current = goal;
            self.previous = Some(goal);
            self.ticks = 0.0;
        } else {
            self.ticks += frames;
            while self.ticks >= 1.0 {
                self.ticks -= 1.0;
                self.previous = Some(*current);
                current.step(goal);
            }
        }
        // Preserve native60Hz chase timing, but render between ticks. Display
        // refresh and VSync jitter must not turn fractional ticks into0/2-step jumps.
        let previous = self.previous.unwrap_or(*current);
        let fraction = if held { 1.0 } else { self.ticks };
        let result = (
            Vec3::from_array(previous.center).lerp(Vec3::from_array(current.center), fraction)
                / 40.0,
            Vec3::from_array(previous.eye()).lerp(Vec3::from_array(current.eye()), fraction) / 40.0,
        );
        if !active && request.is_none() && current.normal_angle() {
            *self = Self::default();
        }
        Some(result)
    }
}

pub(crate) fn talk_view(
    disc: Option<&crate::game_disc::GameDisc>,
    town: &crate::town::Town,
    player: Vec3,
    npc: Vec3,
) -> View {
    // aNPC_draw_data_c.eye_height, not an assumed humanoid eye position.
    let eye = disc
        .and_then(|disc| {
            let table = disc.catalog.get("npc_draw_data_tbl")?;
            let byte = disc
                .rom
                .slice(table.source, table.offset + (238 + 20) * 0x6C + 0x5E, 1)?;
            Some(f32::from(byte[0]))
        })
        .unwrap_or(33.0);
    let low = View::talk(
        (player * 40.0).to_array(),
        (npc * 40.0).to_array(),
        eye,
        false,
    );
    let cliff = town
        .ground
        .as_ref()
        .is_some_and(|ground| ground.talk_camera_high_cliff(Vec3::from_array(low.center) / 40.0));
    View::talk(
        (player * 40.0).to_array(),
        (npc * 40.0).to_array(),
        eye,
        cliff,
    )
}
