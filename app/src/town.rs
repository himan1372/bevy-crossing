use bevy::prelude::*;
use rustimal_logic::town_gen::{self, TownPlan, CellKind, ACRE_WIDTH, ACRE_DEPTH};

/// Size of one acre in world units.
pub const ACRE_SIZE: f32 = 16.0;
/// Size of one cell in world units.
pub const CELL_SIZE: f32 = 1.0;

/// Resource holding the generated town layout.
#[derive(Resource)]
pub struct Town {
    pub plan: TownPlan,
}

impl Town {
    /// Convert acre (ax, az) + cell (cx, cz) to world position.
    pub fn cell_to_world(ax: usize, az: usize, cx: usize, cz: usize) -> Vec3 {
        let wx = (ax as f32 * ACRE_SIZE) + (cx as f32 * CELL_SIZE) + CELL_SIZE / 2.0;
        let wz = (az as f32 * ACRE_SIZE) + (cz as f32 * CELL_SIZE) + CELL_SIZE / 2.0;
        let ox = (ACRE_WIDTH as f32 * ACRE_SIZE) / 2.0;
        let oz = (ACRE_DEPTH as f32 * ACRE_SIZE) / 2.0;
        Vec3::new(wx - ox, 0.0, wz - oz)
    }

    /// Get the cell kind at a world position. Returns None if out of bounds.
    pub fn cell_at(&self, world_pos: Vec3) -> Option<u8> {
        let ox = (ACRE_WIDTH as f32 * ACRE_SIZE) / 2.0;
        let oz = (ACRE_DEPTH as f32 * ACRE_SIZE) / 2.0;
        let lx = world_pos.x + ox;
        let lz = world_pos.z + oz;

        if lx < 0.0 || lz < 0.0 {
            return None;
        }

        let ax = (lx / ACRE_SIZE) as usize;
        let az = (lz / ACRE_SIZE) as usize;
        if ax >= ACRE_WIDTH || az >= ACRE_DEPTH {
            return None;
        }

        let cx = ((lx % ACRE_SIZE) / CELL_SIZE) as usize;
        let cz = ((lz % ACRE_SIZE) / CELL_SIZE) as usize;
        if cx >= 16 || cz >= 16 {
            return None;
        }

        let acre_idx = az * ACRE_WIDTH + ax;
        let cell_idx = cz * 16 + cx;
        Some(self.plan.acres[acre_idx].cells[cell_idx])
    }

    /// Check if a world position is walkable.
    pub fn is_walkable(&self, world_pos: Vec3) -> bool {
        match self.cell_at(world_pos) {
            Some(c) => {
                c == CellKind::Grass as u8
                    || c == CellKind::Flower as u8
                    || c == CellKind::Weed as u8
            }
            None => false,
        }
    }
}

/// Generate a town and store it as a resource.
pub fn generate_town(mut commands: Commands) {
    let pool: Vec<u16> = (0..15).collect();
    let plan = town_gen::generate(12345, &pool, 6)
        .expect("Failed to generate town");

    commands.insert_resource(Town { plan });
    println!("Town generated with seed 12345 (30 acres)");
}

/// Render the town: acres as ground, trees, houses, rocks.
pub fn render_town(
    town: Res<Town>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ground_mat = materials.add(Color::srgb(0.35, 0.7, 0.35));
    let tree_trunk_mat = materials.add(Color::srgb(0.4, 0.25, 0.15));
    let tree_leaf_mat = materials.add(Color::srgb(0.2, 0.6, 0.25));
    let house_mat = materials.add(Color::srgb(0.8, 0.6, 0.4));
    let rock_mat = materials.add(Color::srgb(0.5, 0.5, 0.5));

    let ground_mesh = meshes.add(Plane3d::default().mesh().size(ACRE_SIZE, ACRE_SIZE));
    let trunk_mesh = meshes.add(Cylinder::new(0.3, 1.5));
    let leaf_mesh = meshes.add(Sphere::new(1.2).mesh());
    let house_mesh = meshes.add(Cuboid::new(3.0, 2.5, 3.0));
    let rock_mesh = meshes.add(Sphere::new(0.8).mesh());

    let mut tree_count = 0;
    let mut house_count = 0;

    for az in 0..ACRE_DEPTH {
        for ax in 0..ACRE_WIDTH {
            let acre_idx = az * ACRE_WIDTH + ax;
            let acre = &town.plan.acres[acre_idx];

            let wx = (ax as f32 * ACRE_SIZE) + ACRE_SIZE / 2.0 - (ACRE_WIDTH as f32 * ACRE_SIZE) / 2.0;
            let wz = (az as f32 * ACRE_SIZE) + ACRE_SIZE / 2.0 - (ACRE_DEPTH as f32 * ACRE_SIZE) / 2.0;

            commands.spawn(PbrBundle {
                mesh: ground_mesh.clone(),
                material: ground_mat.clone(),
                transform: Transform::from_xyz(wx, 0.0, wz),
                ..default()
            });

            for cz in 0..16 {
                for cx in 0..16 {
                    let cell_idx = cz * 16 + cx;
                    let cell = acre.cells[cell_idx];
                    let pos = Town::cell_to_world(ax, az, cx, cz);

                    if cell == CellKind::Tree as u8 {
                        tree_count += 1;
                        commands.spawn(PbrBundle {
                            mesh: trunk_mesh.clone(),
                            material: tree_trunk_mat.clone(),
                            transform: Transform::from_xyz(pos.x, 0.75, pos.z),
                            ..default()
                        });
                        commands.spawn(PbrBundle {
                            mesh: leaf_mesh.clone(),
                            material: tree_leaf_mat.clone(),
                            transform: Transform::from_xyz(pos.x, 2.2, pos.z),
                            ..default()
                        });
                    } else if cell == CellKind::House as u8 {
                        house_count += 1;
                        commands.spawn(PbrBundle {
                            mesh: house_mesh.clone(),
                            material: house_mat.clone(),
                            transform: Transform::from_xyz(pos.x, 1.25, pos.z),
                            ..default()
                        });
                    } else if cell == CellKind::Rock as u8 {
                        commands.spawn(PbrBundle {
                            mesh: rock_mesh.clone(),
                            material: rock_mat.clone(),
                            transform: Transform::from_xyz(pos.x, 0.4, pos.z),
                            ..default()
                        });
                    }
                }
            }
        }
    }

    println!("Town rendered: {} trees, {} houses", tree_count, house_count);
}

pub struct TownPlugin;

impl Plugin for TownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generate_town)
            .add_systems(Startup, render_town.after(generate_town));
    }
}
