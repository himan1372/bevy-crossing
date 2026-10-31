//! ac_conveni/super/depart: original posed bodies, palettes and actor offsets.
//! Cache each tier/season once; restore the measured Cranny when loading back.
use super::*;

#[derive(Component)]
pub(super) struct OriginalShop;

struct Model {
    entity: Entity,
    building: RetailBuilding,
    glass: Vec<Handle<N64Material>>,
    glow: Vec<Handle<N64Material>>,
}

#[derive(Default)]
pub(super) struct Cache {
    original: Option<(Entity, RetailBuilding)>,
    models: HashMap<(u8, bool), Model>,
    shown: Option<(u8, bool)>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn update(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    shop: Res<crate::shop::ShopData>,
    location: Res<State<crate::shop::ShopLocation>>,
    mut town: ResMut<Town>,
    mut shop_building: ResMut<crate::shop::ShopBuilding>,
    original: Query<Entity, With<OriginalShop>>,
    mut cache: Local<Cache>,
    mut lights: Option<ResMut<BuildingLights>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    if *location.get() != crate::shop::ShopLocation::Town {
        return;
    }
    let Some(disc) = disc else { return };
    let Some(mut lights) = lights.take() else {
        return;
    };
    let Some(retail) = town.retail.as_ref() else {
        return;
    };
    let Some(index) = retail.buildings.iter().position(|b| b.item == SHOP0) else {
        return;
    };
    if cache.original.is_none() {
        let Ok(entity) = original.single() else {
            return;
        };
        let initial = retail.buildings[index].clone();
        cache.original = Some((entity, initial.clone()));
        let winter = is_winter(current_term());
        cache.models.insert(
            (0, winter),
            Model {
                entity,
                building: initial,
                glass: lights.cranny.glass.clone(),
                glow: lights.cranny.glow.clone(),
            },
        );
        cache.shown = Some((0, winter));
    }
    let key = (shop.shop.shop_level as u8, is_winter(current_term()));
    if cache.shown == Some(key) {
        return;
    }
    if !cache.models.contains_key(&key) {
        let Some((_, baseline)) = cache.original.as_ref() else {
            return;
        };
        let Some(model) = build(
            &mut commands,
            &disc,
            baseline,
            key,
            town.ground_y(baseline.origin),
            (&mut meshes, &mut materials, &mut images),
        ) else {
            return;
        };
        cache.models.insert(key, model);
    }
    for (&model_key, model) in &cache.models {
        crate::intro_camera::Occluder::set_base_visibility(
            &mut commands,
            model.entity,
            if model_key == key {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        );
    }
    let model = &cache.models[&key];
    let building = model.building.clone();
    let base_y = town.ground_y(building.origin);
    let door = building.door.unwrap_or(building.origin);
    let out = (door - building.centre).with_y(0.0).normalize_or(Vec3::Z);
    let inner = building.half.min_element() * std::f32::consts::FRAC_1_SQRT_2;
    *shop_building = crate::shop::ShopBuilding {
        door_pos: (door + out * 0.7).with_y(base_y),
        min_x: building.centre.x - inner,
        max_x: building.centre.x + inner,
        min_z: building.centre.z - inner,
        max_z: building.centre.z + inner,
        placed: true,
        base_y,
        trigger: Some(building.origin + Vec3::new(-38.0, 0.0, 42.0) / 40.0),
    };
    lights.cranny = WindowLights::default();
    lights.cranny.glass = model.glass.clone();
    lights.cranny.glow = model.glow.clone();
    if let Some(retail) = town.retail.as_mut() {
        retail.buildings[index] = building;
    }
    cache.shown = Some(key);
}

fn build(
    commands: &mut Commands,
    disc: &GameDisc,
    baseline: &RetailBuilding,
    (tier, winter): (u8, bool),
    base_y: f32,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Option<Model> {
    if tier == 0 {
        return None;
    }
    let number = tier + 1;
    let season = if winter { "w" } else { "s" };
    let prefix = format!("obj_{season}_shop{number}");
    let skeleton = crate::skeleton::Skeleton::read(
        &disc.rom,
        disc.catalog.get(&format!("cKF_bs_r_{prefix}"))?.offset,
    )?;
    let animation = crate::skeleton::Animation::read(
        &disc.rom,
        disc.catalog.get(&format!("cKF_ba_r_{prefix}"))?.offset,
        skeleton.joints.len(),
    )?;
    let door_lists = part_lists(&disc.catalog, &prefix, &["door", "door1", "door2"]);
    let body_lists = part_lists(&disc.catalog, &prefix, &["t1", "t2", "grass"]);
    if body_lists.is_empty() || door_lists.is_empty() {
        return None;
    }
    let mut all = body_lists.clone();
    all.extend(&door_lists);
    let mut surface = run_display_lists(&disc.rom, &all);
    pose_parts(disc, &all, &mut surface);
    let points: Vec<Vec2> = surface
        .parts
        .iter()
        .flat_map(|p| &p.vertices)
        .map(|v| Vec2::new(v.pos[0], v.pos[2]) * OBJECT_SCALE)
        .collect();
    if points.is_empty() {
        return None;
    }
    let (box_yaw, lo, hi) = tightest_box(&points);
    let mid = (lo + hi) / 2.0;
    // Original actor ct: Cranny(-20,+20), Combini(-40,0), Super/Depart(-20,-20).
    let offset = if tier == 1 {
        Vec3::new(-0.5, 0.0, -0.5)
    } else {
        Vec3::new(0.0, 0.0, -1.0)
    };
    let origin = baseline.origin + offset;
    let door_points: Vec<Vec3> = door_lists
        .iter()
        .flat_map(|list| {
            let mut sink = run_display_lists(&disc.rom, &[*list]);
            pose_parts(disc, &[*list], &mut sink);
            sink.parts
                .iter()
                .flat_map(|p| &p.vertices)
                .map(|v| Vec3::from(v.pos))
                .collect::<Vec<_>>()
        })
        .collect();
    let door = (!door_points.is_empty()).then(|| {
        origin + door_points.iter().sum::<Vec3>() / door_points.len() as f32 * OBJECT_SCALE
    });
    let building = RetailBuilding {
        item: SHOP0,
        spec: baseline.spec,
        origin,
        door,
        centre: origin + Quat::from_rotation_y(box_yaw) * Vec3::new(mid.x, 0.0, mid.y),
        half: (hi - lo) / 2.0,
        box_yaw,
        yaw: 0.0,
        palette_shift: 0,
    };
    let palette = structure_palette(disc, 37 + u32::from(tier), winter);
    let root = commands
        .spawn((
            DiscObject,
            Name::new(prefix.clone()),
            crate::intro_camera::Occluder::new(if tier == 1 { 110.0 } else { 130.0 }),
            Transform::from_translation(origin.with_y(base_y)),
            Visibility::default(),
        ))
        .id();
    let mut model = build_model(disc, &body_lists, palette, meshes, materials, images);
    let mut lights = WindowLights::default();
    model.extend(build_windows(
        disc,
        &prefix,
        "light",
        palette,
        meshes,
        materials,
        images,
        &mut lights,
    ));
    for (mesh, material) in model {
        commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), ChildOf(root)));
    }
    for list in door_lists {
        let joint = skeleton.joints.iter().position(|j| j.model == Some(list))?;
        let door = CrannyDoor {
            skeleton: skeleton.clone(),
            animation: animation.clone(),
            joint,
            frame: 1.0,
        };
        let anchor = commands
            .spawn((
                Name::new("shop exterior door"),
                door.transform(),
                Visibility::default(),
                ChildOf(root),
                door,
            ))
            .id();
        let sink = run_display_lists(&disc.rom, &[list]);
        for (mesh, material) in sink_meshes(
            disc,
            &sink,
            palette,
            false,
            &Segments::default(),
            (meshes, materials, images),
        ) {
            commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), ChildOf(anchor)));
        }
    }
    Some(Model {
        entity: root,
        building,
        glass: lights.glass,
        glow: lights.glow,
    })
}
