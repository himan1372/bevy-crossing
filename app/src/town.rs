use bevy::prelude::*;
use rustimal_logic::town_gen::{self, TownPlan, CellKind, ACRE_WIDTH, ACRE_DEPTH};
use std::collections::HashSet;

/// Size of one acre in world units.
pub const ACRE_SIZE: f32 = 16.0;
/// Size of one cell in world units.
pub const CELL_SIZE: f32 = 1.0;

/// River edge bits, mirroring town_gen's private NORTH/EAST/SOUTH/WEST.
const EDGE_N: u8 = 1;
const EDGE_E: u8 = 2;
const EDGE_S: u8 = 4;
const EDGE_W: u8 = 8;

/// Half-width of the river band in cells (band is 4 cells wide, centered on cell 8).
const RIVER_HALF_WIDTH: usize = 2;

/// Deterministic 0/1 shade jitter for an acre index, so the grass
/// checkerboard has slight organic variation instead of a perfect pattern.
fn acre_jitter(acre_idx: usize) -> usize {
    let mut x = (acre_idx as u32).wrapping_mul(0x9E3779B9).wrapping_add(0x85EBCA6B);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EBCA6B);
    x ^= x >> 13;
    (x % 2) as usize
}

/// Resource holding the generated town layout.
#[derive(Resource)]
pub struct Town {
    pub plan: TownPlan,
    /// (ax, az, cx, cz) cells covered by river water.
    pub river_tiles: HashSet<(usize, usize, usize, usize)>,
}

/// Seed used for town generation. Set from the save file at startup
/// (see `save.rs`); falls back to the default when no save exists.
#[derive(Resource, Clone, Copy)]
pub struct TownSeed(pub u32);

impl Town {
    /// Convert acre (ax, az) + cell (cx, cz) to world position.
    pub fn cell_to_world(ax: usize, az: usize, cx: usize, cz: usize) -> Vec3 {
        let wx = (ax as f32 * ACRE_SIZE) + (cx as f32 * CELL_SIZE) + CELL_SIZE / 2.0;
        let wz = (az as f32 * ACRE_SIZE) + (cz as f32 * CELL_SIZE) + CELL_SIZE / 2.0;
        let ox = (ACRE_WIDTH as f32 * ACRE_SIZE) / 2.0;
        let oz = (ACRE_DEPTH as f32 * ACRE_SIZE) / 2.0;
        Vec3::new(wx - ox, 0.0, wz - oz)
    }

    /// Acre/cell indices at a world position. Returns None if out of bounds.
    fn cell_indices(&self, world_pos: Vec3) -> Option<(usize, usize, usize, usize)> {
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

        Some((ax, az, cx, cz))
    }

    /// Get the cell kind at a world position. Returns None if out of bounds.
    pub fn cell_at(&self, world_pos: Vec3) -> Option<u8> {
        let (ax, az, cx, cz) = self.cell_indices(world_pos)?;

        let acre_idx = az * ACRE_WIDTH + ax;
        let cell_idx = cz * 16 + cx;
        Some(self.plan.acres[acre_idx].cells[cell_idx])
    }

    /// Check if a world position is a river water tile.
    pub fn is_river(&self, world_pos: Vec3) -> bool {
        match self.cell_indices(world_pos) {
            Some((ax, az, cx, cz)) => self.river_tiles.contains(&(ax, az, cx, cz)),
            None => false,
        }
    }

    /// Check if a world position is walkable.
    pub fn is_walkable(&self, world_pos: Vec3) -> bool {
        let (ax, az, cx, cz) = match self.cell_indices(world_pos) {
            Some(i) => i,
            None => return false,
        };
        // River water is never walkable.
        if self.river_tiles.contains(&(ax, az, cx, cz)) {
            return false;
        }
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

/// Trace the river path as a list of (ax, az) acre coords in flow order,
/// following the river_edges bits from the northern start to the ocean outlet.
fn trace_river_path(plan: &TownPlan) -> Vec<(usize, usize)> {
    // The river always starts on the top row, flowing south.
    let mut start: Option<(i32, i32)> = None;
    for ax in 0..ACRE_WIDTH {
        if plan.acres[ax].river_edges & EDGE_S != 0 {
            start = Some((ax as i32, 0));
            break;
        }
    }
    let mut path = Vec::new();
    let (mut x, mut z) = match start {
        Some(p) => p,
        None => return path,
    };
    let mut prev: Option<(i32, i32)> = None;
    // (edge bit, dx, dz)
    const DIRS: [(u8, i32, i32); 4] = [
        (EDGE_N, 0, -1),
        (EDGE_E, 1, 0),
        (EDGE_S, 0, 1),
        (EDGE_W, -1, 0),
    ];
    loop {
        path.push((x as usize, z as usize));
        let acre = &plan.acres[z as usize * ACRE_WIDTH + x as usize];
        let mut next: Option<(i32, i32)> = None;
        for &(bit, dx, dz) in &DIRS {
            if acre.river_edges & bit == 0 {
                continue;
            }
            let (nx, nz) = (x + dx, z + dz);
            if Some((nx, nz)) == prev {
                continue; // don't flow back where we came from
            }
            if nx < 0 || nz < 0 || nx >= ACRE_WIDTH as i32 || nz >= ACRE_DEPTH as i32 {
                continue; // ocean outlet / map edge
            }
            next = Some((nx, nz));
            break;
        }
        match next {
            Some((nx, nz)) => {
                prev = Some((x, z));
                x = nx;
                z = nz;
            }
            None => break,
        }
    }
    path
}

/// Which side of an acre faces the neighbor at (bx, bz).
fn side_toward(ax: usize, az: usize, bx: usize, bz: usize) -> u8 {
    if bx > ax {
        EDGE_E
    } else if bx < ax {
        EDGE_W
    } else if bz > az {
        EDGE_S
    } else {
        EDGE_N
    }
}

/// Water cells (cx, cz) within one river acre, given the entry and exit sides.
/// Draws a 4-cell-wide band: straight through for N<->S / E<->W,
/// an L-shape through the center for turns.
fn band_cells(entry: u8, exit: u8) -> Vec<(usize, usize)> {
    let lo = 8 - RIVER_HALF_WIDTH; // 6
    let hi = 8 + RIVER_HALF_WIDTH; // 10 (exclusive)
    let mut cells = Vec::new();
    let is_vertical = |s: u8| s == EDGE_N || s == EDGE_S;
    if is_vertical(entry) && is_vertical(exit) {
        for cz in 0..16 {
            for cx in lo..hi {
                cells.push((cx, cz));
            }
        }
    } else if !is_vertical(entry) && !is_vertical(exit) {
        for cz in lo..hi {
            for cx in 0..16 {
                cells.push((cx, cz));
            }
        }
    } else {
        let v_side = if is_vertical(entry) { entry } else { exit };
        let h_side = if is_vertical(entry) { exit } else { entry };
        // Vertical arm: from the vertical edge to the center band.
        let (cz0, cz1) = if v_side == EDGE_N { (0, hi) } else { (lo, 16) };
        for cz in cz0..cz1 {
            for cx in lo..hi {
                cells.push((cx, cz));
            }
        }
        // Horizontal arm: from the horizontal edge to the center band.
        let (cx0, cx1) = if h_side == EDGE_W { (0, hi) } else { (lo, 16) };
        for cz in lo..hi {
            for cx in cx0..cx1 {
                cells.push((cx, cz));
            }
        }
    }
    cells
}

/// Compute every (ax, az, cx, cz) tile covered by river water.
fn river_water_tiles(plan: &TownPlan) -> HashSet<(usize, usize, usize, usize)> {
    let mut tiles = HashSet::new();
    let path = trace_river_path(plan);
    for (i, &(ax, az)) in path.iter().enumerate() {
        // Entry side: from the previous acre, or the northern map edge for the start.
        let entry = if i > 0 {
            let (px, pz) = path[i - 1];
            side_toward(ax, az, px, pz)
        } else {
            EDGE_N
        };
        // Exit side: toward the next acre, or the southern ocean outlet for the end.
        let exit = if i + 1 < path.len() {
            let (nx, nz) = path[i + 1];
            side_toward(ax, az, nx, nz)
        } else {
            EDGE_S
        };
        for (cx, cz) in band_cells(entry, exit) {
            tiles.insert((ax, az, cx, cz));
        }
    }
    tiles
}

/// Generate a town and store it as a resource.
/// Uses `TownSeed` when the save system has set one (PreStartup),
/// otherwise falls back to the default seed.
pub fn generate_town(mut commands: Commands, seed: Option<Res<TownSeed>>) {
    let seed_val = seed.map(|s| s.0).unwrap_or(12345);
    let pool: Vec<u16> = (0..15).collect();
    let plan = town_gen::generate(seed_val, &pool, 6)
        .expect("Failed to generate town");

    let river_tiles = river_water_tiles(&plan);
    println!(
        "Town generated with seed {seed_val} (30 acres), river covers {} tiles",
        river_tiles.len()
    );
    commands.insert_resource(Town { plan, river_tiles });
}

/// Render the town: acres as ground, trees, houses, rocks.
pub fn render_town(
    town: Res<Town>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Grass: two complementary greens in an acre-level checkerboard (like the
    // retail games), each with a subtle deterministic shade variant so the
    // town doesn't look perfectly uniform. Four materials, zero extra meshes.
    let grass_mats = [
        materials.add(Color::srgb(0.36, 0.72, 0.34)), // light
        materials.add(Color::srgb(0.33, 0.69, 0.31)), // light, slightly muted
        materials.add(Color::srgb(0.30, 0.65, 0.29)), // dark
        materials.add(Color::srgb(0.28, 0.62, 0.27)), // dark, slightly muted
    ];
    let water_mat = materials.add(Color::srgb(0.22, 0.5, 0.88));
    let tree_trunk_mat = materials.add(Color::srgb(0.4, 0.25, 0.15));
    let tree_leaf_mat = materials.add(Color::srgb(0.2, 0.6, 0.25));
    let house_mat = materials.add(Color::srgb(0.8, 0.6, 0.4));
    let rock_mat = materials.add(Color::srgb(0.5, 0.5, 0.5));

    let ground_mesh = meshes.add(Plane3d::default().mesh().size(ACRE_SIZE, ACRE_SIZE));
    let water_mesh = meshes.add(Plane3d::default().mesh().size(CELL_SIZE, CELL_SIZE));
    let trunk_mesh = meshes.add(Cylinder::new(0.3, 1.5));
    let leaf_mesh = meshes.add(Sphere::new(1.2).mesh());
    let house_mesh = meshes.add(Cuboid::new(3.0, 2.5, 3.0));
    let rock_mesh = meshes.add(Sphere::new(0.8).mesh());

    let mut tree_count = 0;
    let mut house_count = 0;
    let mut water_count = 0;

    for az in 0..ACRE_DEPTH {
        for ax in 0..ACRE_WIDTH {
            let acre_idx = az * ACRE_WIDTH + ax;
            let acre = &town.plan.acres[acre_idx];

            let wx = (ax as f32 * ACRE_SIZE) + ACRE_SIZE / 2.0 - (ACRE_WIDTH as f32 * ACRE_SIZE) / 2.0;
            let wz = (az as f32 * ACRE_SIZE) + ACRE_SIZE / 2.0 - (ACRE_DEPTH as f32 * ACRE_SIZE) / 2.0;

            // Checkerboard grass: alternate light/dark acres, with a deterministic
            // per-acre shade jitter for a less artificial look.
            let checker = (ax + az) % 2;
            let jitter = acre_jitter(acre_idx);
            let grass_mat = grass_mats[checker * 2 + jitter].clone();

            commands.spawn(PbrBundle {
                mesh: ground_mesh.clone(),
                material: grass_mat,
                transform: Transform::from_xyz(wx, 0.0, wz),
                ..default()
            });

            for cz in 0..16 {
                for cx in 0..16 {
                    let pos = Town::cell_to_world(ax, az, cx, cz);

                    // River water: flat blue plane, no decorations on top.
                    if town.river_tiles.contains(&(ax, az, cx, cz)) {
                        water_count += 1;
                        commands.spawn(PbrBundle {
                            mesh: water_mesh.clone(),
                            material: water_mat.clone(),
                            transform: Transform::from_xyz(pos.x, 0.05, pos.z),
                            ..default()
                        });
                        continue;
                    }

                    let cell_idx = cz * 16 + cx;
                    let cell = acre.cells[cell_idx];

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

    println!(
        "Town rendered: {} trees, {} houses, {} water tiles",
        tree_count, house_count, water_count
    );
}

pub struct TownPlugin;

impl Plugin for TownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generate_town)
            .add_systems(Startup, render_town.after(generate_town));
    }
}
