//! Reload an existing house's roof palette without touching shared neighbours.
use super::*;

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

#[derive(Component)]
pub(super) struct Appearance {
    size: u8,
    palette: u8,
    winter: bool,
}

/// At most twelve palettes for the currently built house size. Cached window
/// materials also stay in BuildingLights, so reusing these keeps that bounded.
#[derive(Resource, Default)]
pub(super) struct Cache(HashMap<(u8, u8), Model>);

pub(super) fn track(
    commands: &mut Commands,
    root: Entity,
    (size, palette): (u8, u8),
    winter: bool,
    model: &Model,
) {
    commands.entity(root).insert(Appearance {
        size,
        palette,
        winter,
    });
    commands.insert_resource(Cache(HashMap::from([((size, palette), model.clone())])));
}

#[allow(clippy::too_many_arguments)] // Rebuild all native house parts with one palette and lighting cache.
pub(super) fn update(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    mut roots: Query<(Entity, &mut Appearance, &Children)>,
    mut cache: ResMut<Cache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    lights: Option<ResMut<BuildingLights>>,
) {
    let (Some(disc), Some(mut lights)) = (disc, lights) else {
        return;
    };
    let wanted = crate::home::look();
    for (root, mut shown, children) in &mut roots {
        // Preserve the existing restart requirement for changing house size.
        if shown.size != wanted.0 || shown.palette == wanted.1 {
            continue;
        }
        if let std::collections::hash_map::Entry::Vacant(entry) = cache.0.entry(wanted) {
            let season = if shown.winter { "w" } else { "s" };
            let Some(model) = player_home_model(
                &disc,
                season,
                wanted,
                &mut meshes,
                &mut materials,
                &mut images,
                &mut lights,
            ) else {
                continue;
            };
            entry.insert(model);
            lights.normal.invalidate();
        }
        for child in children.iter() {
            commands.entity(child).try_despawn();
        }
        for (mesh, material) in &cache.0[&wanted] {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                ChildOf(root),
            ));
        }
        shown.palette = wanted.1;
        info!("player house roof palette refreshed: {}", wanted.1);
    }
}
