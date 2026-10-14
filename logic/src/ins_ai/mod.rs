//! Insect behaviour: the shared per-tick work of `ac_insect_move.c_inc`
//! (patience from nearby movement, position move) and the families of
//! `ac_ins_*.c`. Game units, one step per 60 Hz tick.
//!
//! Families so far: butterflies (`ac_ins_chou.c`), cicadas
//! (`ac_ins_semi.c`), tree beetles (`ac_ins_kabuto.c`),
//! grasshoppers and crickets (`ac_ins_batta.c`), mosquitoes
//! (`ac_ins_ka.c`), pondskaters (`ac_ins_amenbo.c`), dragonflies
//! (`ac_ins_tonbo.c`).

mod amenbo;
mod batta;
mod ka;
mod kabuto;
mod semi;
mod tonbo;

use crate::gyo_ai::{angle_to, chase_angle, chase_f, cos_s, sin_s};

/// `aINS_INSECT_TYPE_*` of the butterflies.
pub const COMMON_BUTTERFLY: u8 = 0;
pub const YELLOW_BUTTERFLY: u8 = 1;
pub const TIGER_BUTTERFLY: u8 = 2;
pub const PURPLE_BUTTERFLY: u8 = 3;

/// One unit (`mFI_UNIT_BASE_SIZE_F`).
const UNIT: f32 = 40.0;
/// `aINS_MAX_STRESS_DIST`, `aINS_PATIENCE_STEP`.
const MAX_STRESS_DIST: f32 = 3.0 * UNIT;
const PATIENCE_STEP: f32 = 0.5;
/// `catch_ME_data`: per type, added to the stress distance.
const CATCH_ME: [f32; 41] = [
    0.0, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0, 0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 20.0, -20.0, -20.0,
    -20.0, -20.0, -20.0, -20.0, -20.0, 0.0, 0.0, -20.0, -20.0, -20.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

/// `FLOWER_PANSIES0`: the one item `aICH_rest_check` lands on.
const FLOWER_PANSIES0: u16 = 0x0845;
const FLOWER_TULIP2: u16 = 0x084D;

/// What an insect asks of the town.
pub trait Field {
    fn ground_y(&self, x: f32, z: f32) -> f32;
    /// Water surface, if the point is over water.
    fn water_y(&self, x: f32, z: f32) -> Option<f32>;
    /// (acre x, acre z, unit x, unit z) under a point.
    fn unit_of(&self, x: f32, z: f32) -> Option<[i32; 4]>;
    fn unit_center(&self, u: [i32; 4]) -> [f32; 2];
    /// The field item on a unit (`None`: no data, or reserved).
    fn item(&self, u: [i32; 4]) -> Option<u16>;
    /// A wall between two points at this height (BG front hit).
    fn blocked(&self, from: [f32; 3], to: [f32; 3]) -> bool;
    /// Collision attribute under a point.
    fn attribute(&self, x: f32, z: f32) -> Option<u8>;
    /// Top of the column on a unit, if one stands there
    /// (`mCoBG_Wpos2BgUtCenterHeight_AddColumn`).
    fn column_top(&self, _u: [i32; 4]) -> Option<f32> {
        None
    }
}

/// Something that moves near insects (player, villagers): position and
/// how far it went on the xz plane this tick.
#[derive(Clone, Copy, Debug)]
pub struct Mover {
    pub pos: [f32; 3],
    pub moved: f32,
}

/// The tick's surroundings.
pub struct World<'a> {
    pub player: [f32; 3],
    /// `shape_info.rotation.y` of the player.
    pub player_facing: i16,
    /// Player first, then villagers.
    pub movers: &'a [Mover],
    /// Where a net swing came down (`mPlib_Check_StopNet`).
    pub net_stop: Option<[f32; 2]>,
    /// Where a shovel dug (`mPlib_Check_DigScoop`).
    pub scoop: Option<[f32; 2]>,
    /// A tree being shaken (`mPlib_Check_tree_shaken`).
    pub tree_shaken: Option<[f32; 2]>,
    /// A unit an axe hit this tick (`mPlib_Check_VibUnit_OneFrame`).
    pub axe_hit: Option<[f32; 2]>,
    pub raining: bool,
    /// A cutscene or chore is on (`demo_clip`): mosquitoes leave.
    pub busy: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Avoid,
    LetEscape,
    Fly,
    Landing,
    Hover,
    Rest,
    /// Cicada on its tree; dragonfly turning before it darts again.
    Wait,
    /// The banded dragonfly's endless patrol.
    Patrol,
    TouchWater,
    TouchWaterUp,
    HoverOnWater,
    FlyToRest,
    RestOnSpot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    None,
    /// Faded out after flying off (or drowned): remove it.
    Gone,
    /// A mosquito stung the player.
    Stung,
}

#[derive(Clone, Debug)]
pub struct Insect {
    pub kind: u8,
    pub pos: [f32; 3],
    pub angle: i16,
    pub action: Action,
    /// 0..100; 90+ and it's off.
    pub patience: f32,
    /// Alpha 0..255 while fading out.
    pub alpha: u8,
    speed: f32,
    target_speed: f32,
    speed_step: f32,
    vy: f32,
    gravity: f32,
    max_vy: f32,
    /// `_1E0`: wing frame 0..2.
    anim: f32,
    /// `flag`: fuwafuwa phase.
    phase: i16,
    /// `_254`: avoid swerve phase, 0..64.
    swerve: f32,
    acre: [i32; 2],
    /// `s32_work1/2`: the unit it circles; `f32_work0/1`: its centre.
    unit: [i32; 2],
    centre: [f32; 2],
    /// `s32_work0`, `s32_work3`: loop corner and its latch.
    corner: usize,
    latch: bool,
    /// `f32_work2`: wall search cooldown; `f32_work3`: rest countdown.
    wall_cool: f32,
    rest_cool: f32,
    /// `light_flag`: which landing spot.
    spot: usize,
    timer: f32,
    alpha_time: f32,
    wall_front: bool,
    /// `shape_info.rotation`: which way it faces, and its pitch (a
    /// cicada is stood up on the trunk).
    pub facing: i16,
    pub pitch: i16,
    /// Flying off and fading (`life_time` 0, `alpha_time` counting).
    fading: bool,
    on_ground: bool,
    home: [f32; 3],
    /// Dragonflies: cruise height, rest height, acre centre, stop kind,
    /// wall hits in a row, water dips, the 100-tick phase.
    height: f32,
    rest_y: f32,
    centre_bk: [f32; 2],
    stop: u8,
    hits: u32,
    dips: u8,
    phase100: u32,
    /// Kept this far over the floor by the BG check (`-bg_height`).
    floor: f32,
}

impl Insect {
    /// `aICH_actor_init` for a placed butterfly; `None` when it can't live
    /// there (`aICH_check_live_condition`).
    pub fn butterfly(
        kind: u8,
        pos: [f32; 2],
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        if kind != PURPLE_BUTTERFLY {
            match field.item(u) {
                None => return None,
                Some(i) if (i >> 12) & 0xF == 5 => return None,
                _ => {}
            }
        }
        // GetBgY_OnlyCenter(pos, 30): 30 under; jump_ctrl lifts it.
        let y = field.ground_y(pos[0], pos[1]) - 30.0;
        let mut ins = Insect::blank(kind, [pos[0], y, pos[1]], u);
        ins.setup(Action::Fly, None, field, rand);
        Some(ins)
    }

    /// Families with a behaviour here.
    pub fn supported(kind: u8) -> bool {
        kind <= PURPLE_BUTTERFLY
            || semi::is_cicada(kind)
            || kabuto::is_beetle(kind)
            || tonbo::is_dragonfly(kind)
            || batta::is_hopper(kind)
            || kind == ka::MOSQUITO
            || kind == amenbo::POND_SKATER
    }

    /// A placed insect of any supported family; `None` when it can't live
    /// there (or isn't supported).
    pub fn new(
        kind: u8,
        pos: [f32; 2],
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Option<Self> {
        if kind <= PURPLE_BUTTERFLY {
            Self::butterfly(kind, pos, field, rand)
        } else if semi::is_cicada(kind) || kabuto::is_beetle(kind) {
            Self::on_trunk(kind, pos, field)
        } else if kind == ka::MOSQUITO {
            Self::mosquito(pos, field)
        } else if kind == amenbo::POND_SKATER {
            Self::pond_skater(pos, field, rand)
        } else if batta::is_hopper(kind) {
            Self::hopper(kind, pos, field, rand)
        } else if tonbo::is_dragonfly(kind) {
            Self::dragonfly(kind, pos, field, rand)
        } else {
            None
        }
    }

    fn blank(kind: u8, pos: [f32; 3], u: [i32; 4]) -> Self {
        Insect {
            kind,
            pos,
            angle: 0,
            action: Action::Wait,
            patience: 0.0,
            alpha: 255,
            speed: 0.0,
            target_speed: 0.0,
            speed_step: 0.0,
            vy: 0.0,
            gravity: 0.0,
            max_vy: 0.0,
            anim: 0.0,
            phase: 0,
            swerve: 0.0,
            acre: [u[0], u[1]],
            unit: [u[2], u[3]],
            centre: [pos[0], pos[2]],
            corner: 0,
            latch: false,
            wall_cool: 0.0,
            rest_cool: 0.0,
            spot: 0,
            timer: 0.0,
            alpha_time: 0.0,
            wall_front: false,
            facing: 0,
            pitch: 0,
            fading: false,
            on_ground: false,
            home: pos,
            height: 0.0,
            rest_y: 0.0,
            centre_bk: [0.0; 2],
            stop: 0,
            hits: 0,
            dips: 0,
            phase100: 0,
            floor: 0.0,
        }
    }

    /// Fly off fading, as every family's let-escape and avoid start
    /// (`life_time = 0`, `alpha_time = 80`).
    fn start_fading(&mut self) {
        self.fading = true;
        self.alpha_time = 80.0;
    }

    /// Wing frame (`(int)_1E0`).
    pub fn frame(&self) -> usize {
        self.anim as usize % 2
    }

    /// One 60 Hz tick (`aINS_actor_move` then `aICH_actor_move`).
    pub fn step(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Event {
        self.position_move(field);
        self.calc_patience(world);
        if self.fading && self.fade() {
            return Event::Gone;
        }
        if semi::is_cicada(self.kind) {
            self.cicada_step(world, rand);
            return Event::None;
        }
        if self.kind == ka::MOSQUITO {
            return self.mosquito_step(world, field, rand);
        }
        if self.kind == amenbo::POND_SKATER {
            self.skater_step(world, field, rand);
            return Event::None;
        }
        if batta::is_hopper(self.kind) {
            return if self.hopper_step(world, field, rand) {
                Event::Gone
            } else {
                Event::None
            };
        }
        if kabuto::is_beetle(self.kind) {
            self.beetle_step(world, rand);
            return Event::None;
        }
        if tonbo::is_dragonfly(self.kind) {
            self.dragonfly_step(world, field, rand);
            return Event::None;
        }
        self.check_block_edge(world, field, rand);
        match self.action {
            Action::Avoid => self.avoid(world, field, rand),
            Action::LetEscape => self.let_escape(),
            Action::Fly => self.fly(world, field, rand),
            Action::Landing => self.landing(world, field, rand),
            Action::Hover | Action::Rest => self.hover(world, field, rand),
            _ => {}
        }
        self.facing = self.angle;
        Event::None
    }

    /// `aINS_position_move` (butterflies, fireflies, mosquitoes and the
    /// spirit take no gravity) with the BG wall and floor check.
    fn position_move(&mut self, field: &impl Field) {
        chase_f(&mut self.speed, self.target_speed, self.speed_step * 0.5);
        let butterfly = self.kind <= PURPLE_BUTTERFLY;
        if !butterfly && !matches!(self.kind, 27 | ka::MOSQUITO | 40) {
            chase_f(&mut self.vy, self.max_vy, self.gravity * 0.5);
        }
        let to = [
            self.pos[0] + self.speed * sin_s(self.angle) * 0.5,
            self.pos[1] + self.vy * 0.5,
            self.pos[2] + self.speed * cos_s(self.angle) * 0.5,
        ];
        self.wall_front = field.blocked(self.pos, to);
        if self.wall_front {
            self.pos[1] = to[1];
        } else {
            self.pos = to;
        }
        // BG check with ground (dragonflies; a flying-off cicada too).
        self.on_ground = false;
        if !butterfly
            && self.kind != amenbo::POND_SKATER
            && !((semi::is_cicada(self.kind) || kabuto::is_beetle(self.kind))
                && self.action == Action::Wait)
        {
            let g = field.ground_y(self.pos[0], self.pos[2]) + self.floor;
            if self.pos[1] < g {
                self.pos[1] = g;
                self.on_ground = true;
            }
        }
    }

    /// `aINS_calc_patience`: the worst stress of anything moving nearby.
    fn calc_patience(&mut self, world: &World) {
        const TABLE: [f32; 5] = [0.3, 1.0, 2.0, 5.0, 10.0];
        let max = MAX_STRESS_DIST + CATCH_ME.get(self.kind as usize).copied().unwrap_or(0.0);
        let mut stress = 0.0f32;
        for m in world.movers {
            let d = ((m.pos[0] - self.pos[0]).powi(2)
                + (m.pos[1] - self.pos[1]).powi(2)
                + (m.pos[2] - self.pos[2]).powi(2))
            .sqrt();
            if d < max {
                let near = (d - UNIT).max(0.0);
                let idx = ((((max - UNIT) - near) as i32) / 20).clamp(0, 4) as usize;
                stress = stress.max(m.moved * TABLE[idx]);
            }
        }
        if stress == 0.0 {
            self.patience = (self.patience - PATIENCE_STEP).max(0.0);
        } else {
            self.patience = (self.patience + stress * PATIENCE_STEP).min(100.0);
        }
    }

    /// `aINS_calc_alpha_time` while flying off; true once invisible.
    fn fade(&mut self) -> bool {
        if self.alpha_time > 0.0 {
            self.alpha_time -= 1.0;
            let fade = 24.0 - self.alpha_time;
            if fade > 0.0 {
                let a = 255 - (fade * 11.0) as i32;
                self.alpha = a.max(0) as u8;
                return a <= 0;
            }
        }
        false
    }

    fn setup(
        &mut self,
        action: Action,
        world: Option<&World>,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        self.action = action;
        match action {
            Action::Avoid => {
                self.target_speed = 3.0;
                self.speed_step = 0.3;
                self.speed = 2.0;
            }
            Action::LetEscape => {
                self.start_fading();
                self.phase = 0;
                self.speed = 1.5;
                if let Some(w) = world {
                    self.angle = w
                        .player_facing
                        .wrapping_add((21845.0 * (rand(1.0) - 0.5)) as i16);
                }
            }
            Action::Fly => {
                self.gravity = 0.3;
                self.max_vy = -2.0;
                self.target_speed = 2.0;
                self.speed_step = 0.2;
                self.corner = 0;
                self.latch = false;
                self.centre = [self.pos[0], self.pos[2]];
                self.wall_cool = 0.0;
                self.rest_cool = 120.0;
                self.flower_search(field, rand);
            }
            Action::Landing => {
                self.spot = (rand(3.0) as usize).min(2);
                self.vy = 0.0;
            }
            Action::Hover => {
                self.target_speed = 0.0;
                self.speed_step = 0.0;
                self.speed = 0.0;
                self.timer = 10.0;
            }
            Action::Rest => {
                self.anim = 1.0;
                self.timer = 0.5 * (30.0 + rand(60.0));
            }
            _ => {}
        }
    }

    /// `aICH_anime_proc`.
    fn flap(&mut self) {
        self.anim += 0.2;
        if self.anim >= 2.0 {
            self.anim -= 2.0;
        }
    }

    /// `aICH_jump_ctrl`: hop up when below the floor (`-base` over ground
    /// or water), otherwise sink toward `max_vy`.
    fn jump_ctrl(&mut self, base: f32, field: &impl Field, rand: &mut impl FnMut(f32) -> f32) {
        let (x, z) = (self.pos[0], self.pos[2]);
        let floor = field.water_y(x, z).unwrap_or_else(|| field.ground_y(x, z)) - base;
        if self.pos[1] < floor {
            self.vy = 2.0 + rand(1.0);
        }
        chase_f(&mut self.vy, self.max_vy, 0.5 * self.gravity);
    }

    fn in_unit(&self, u: [i32; 2]) -> [i32; 4] {
        [self.acre[0], self.acre[1], u[0], u[1]]
    }

    /// `aICH_flower_search_sub`: move the loop to a neighbouring unit
    /// (`type` 1: one it could sit on; 2: two away; 0: any).
    fn flower_search_sub(
        &mut self,
        kind: u8,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> bool {
        let reach = if kind == 2 { 2 } else { 1 };
        let dx = ((rand(3.0) as i32).min(2) - 1) * reach;
        let dz = ((rand(3.0) as i32).min(2) - 1) * reach;
        let mut u = [self.unit[0] + dx, self.unit[1] + dz];
        if !(1..=14).contains(&u[0]) {
            u[0] = self.unit[0];
        }
        if !(1..=14).contains(&u[1]) {
            u[1] = self.unit[1];
        }
        let full = self.in_unit(u);
        if kind != 0 && !field.item(full).is_some_and(npc_on) {
            return false;
        }
        let c = field.unit_center(full);
        if field.ground_y(c[0], c[1]) - self.pos[1] < 20.0 {
            self.unit = u;
            self.centre = c;
            true
        } else if kind != 2 {
            self.flower_search_sub(2, field, rand)
        } else {
            false
        }
    }

    /// `aICH_flower_search`: half the time, or when not over a flower.
    fn flower_search(&mut self, field: &impl Field, rand: &mut impl FnMut(f32) -> f32) {
        let on_flower = field
            .unit_of(self.pos[0], self.pos[2])
            .and_then(|u| field.item(u))
            .is_some_and(|i| (FLOWER_PANSIES0..=FLOWER_TULIP2).contains(&i));
        if (rand(1.0) >= 0.5 || !on_flower) && self.kind != PURPLE_BUTTERFLY {
            self.flower_search_sub(1, field, rand);
        }
    }

    /// `aICH_loop_move_ctrl`: circle the centre through four corners.
    fn loop_move(&mut self, field: &impl Field, rand: &mut impl FnMut(f32) -> f32) {
        const PT_X: [f32; 4] = [10.0, 10.0, -10.0, -10.0];
        const PT_Z: [f32; 4] = [10.0, -10.0, 10.0, -10.0];
        const CHK_X: [f32; 4] = [-1.0, 1.0, 1.0, -1.0];
        const CHK_Z: [f32; 4] = [-1.0, 1.0, -1.0, 1.0];
        let i = self.corner;
        let x = self.pos[0] - (self.centre[0] + PT_X[i]);
        let z = self.pos[2] - (self.centre[1] + PT_Z[i]);
        let step = if (x * x + z * z).sqrt() < 15.0 {
            0x800
        } else {
            0x400
        };
        chase_angle(&mut self.angle, angle_to([0.0, 0.0], [-x, -z]), step);
        if x * CHK_X[i] < 0.0 || z * CHK_Z[i] < 0.0 {
            if !self.latch {
                self.latch = true;
                self.corner += 1;
                if self.corner > 3 {
                    self.corner = 0;
                    self.flower_search(field, rand);
                }
            }
        } else {
            self.latch = false;
        }
    }

    /// `aICH_avoid_player` / `aICH_avoid_move_ctrl`: away from the player,
    /// swerving; back toward the acre's middle near its edge.
    fn avoid_move(&mut self, world: &World, field: &impl Field) {
        let edge = field
            .unit_of(self.pos[0], self.pos[2])
            .is_some_and(|u| !(1..15).contains(&u[2]) || !(1..15).contains(&u[3]));
        let target = if edge {
            let c = field.unit_center([self.acre[0], self.acre[1], 8, 8]);
            angle_to(
                [self.pos[0], self.pos[2]],
                [c[0] - UNIT / 2.0, c[1] - UNIT / 2.0],
            )
        } else {
            self.swerve = (self.swerve + 1.0) % 64.0;
            let away = angle_to(
                [self.pos[0], self.pos[2]],
                [world.player[0], world.player[2]],
            )
            .wrapping_add(i16::MIN);
            away.wrapping_add(if self.swerve >= 32.0 { 0x1000 } else { -0x1000 })
        };
        chase_angle(&mut self.angle, target, 0x600);
    }

    /// `aICH_check_patience`: a ball, net or shovel close by maxes it.
    fn check_patience(&mut self, world: &World) -> bool {
        let near = |p: Option<[f32; 2]>| {
            p.is_some_and(|p| (p[0] - self.pos[0]).powi(2) + (p[1] - self.pos[2]).powi(2) < 3600.0)
        };
        if near(world.net_stop) || near(world.scoop) {
            self.patience = 100.0;
        }
        self.patience >= 90.0
    }

    /// `aICH_check_block_edge`: past the acre's outer ring, it leaves.
    fn check_block_edge(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        if self.action == Action::LetEscape {
            return;
        }
        let inside = field.unit_of(self.pos[0], self.pos[2]).is_some_and(|u| {
            [u[0], u[1]] == self.acre && (1..=14).contains(&u[2]) && (1..=14).contains(&u[3])
        });
        if !inside {
            self.setup(Action::LetEscape, Some(world), field, rand);
        }
    }

    fn avoid(&mut self, world: &World, field: &impl Field, rand: &mut impl FnMut(f32) -> f32) {
        self.flap();
        let base = match self.kind {
            TIGER_BUTTERFLY => -80.0,
            PURPLE_BUTTERFLY => -100.0,
            _ => -60.0,
        };
        self.jump_ctrl(base, field, rand);
        self.avoid_move(world, field);
        if self.patience < 70.0 {
            if let Some(u) = field.unit_of(self.pos[0], self.pos[2]) {
                self.unit = [u[2], u[3]];
            }
            self.setup(Action::Fly, Some(world), field, rand);
        }
    }

    /// `aICH_let_escape` with `aICH_chou_fuwafuwa`: bob up and away.
    fn let_escape(&mut self) {
        let before = 10.0 * sin_s(self.phase);
        let after = 10.0 * sin_s(self.phase.wrapping_add(0x800));
        self.phase = self.phase.wrapping_add(0x800);
        self.vy = self.gravity + (after - before);
        self.gravity += 0.1;
        self.flap();
    }

    fn fly(&mut self, world: &World, field: &impl Field, rand: &mut impl FnMut(f32) -> f32) {
        self.flap();
        // aICH_BGcheck: a wall ahead sends it to another unit.
        if self.wall_front && self.wall_cool <= 0.0 {
            let kind = if matches!(self.kind, TIGER_BUTTERFLY | PURPLE_BUTTERFLY) {
                0
            } else {
                1
            };
            if self.flower_search_sub(kind, field, rand) {
                self.wall_cool = 10.0;
            }
        }
        self.wall_cool = (self.wall_cool - 1.0).max(0.0);
        let base = match self.kind {
            TIGER_BUTTERFLY => -40.0,
            PURPLE_BUTTERFLY => -50.0,
            _ => -30.0,
        };
        self.jump_ctrl(base, field, rand);
        self.loop_move(field, rand);
        if matches!(self.kind, TIGER_BUTTERFLY | PURPLE_BUTTERFLY) && self.patience > 90.0 {
            self.setup(Action::Avoid, Some(world), field, rand);
        } else if self.kind != PURPLE_BUTTERFLY {
            self.rest_check(world, field, rand);
        }
    }

    /// `aICH_rest_check`: after its countdown, over a white pansy it lands.
    fn rest_check(&mut self, world: &World, field: &impl Field, rand: &mut impl FnMut(f32) -> f32) {
        if self.rest_cool == 0.0 {
            let on = field
                .unit_of(self.pos[0], self.pos[2])
                .and_then(|u| field.item(u));
            if on == Some(FLOWER_PANSIES0) {
                self.setup(Action::Landing, Some(world), field, rand);
            }
        } else {
            self.rest_cool = (self.rest_cool - 0.5).max(0.0);
        }
    }

    fn scared_action(&self) -> Action {
        if matches!(self.kind, TIGER_BUTTERFLY | PURPLE_BUTTERFLY) {
            Action::Avoid
        } else {
            Action::Fly
        }
    }

    fn landing(&mut self, world: &World, field: &impl Field, rand: &mut impl FnMut(f32) -> f32) {
        const SPOT: [[f32; 2]; 3] = [[-11.0, -4.0], [10.0, -8.0], [2.0, 14.0]];
        self.flap();
        if self.check_patience(world) {
            self.setup(self.scared_action(), Some(world), field, rand);
            return;
        }
        let c = field.unit_center(self.in_unit(self.unit));
        let to = [c[0] + SPOT[self.spot][0], c[1] + SPOT[self.spot][1]];
        let a = angle_to([self.pos[0], self.pos[2]], to);
        chase_angle(&mut self.angle, a, 0x1000);
        if self.target_speed == 0.0
            || ((self.pos[0] - to[0]).abs() < 2.0 && (self.pos[2] - to[1]).abs() < 2.0)
        {
            self.setup(Action::Hover, Some(world), field, rand);
        }
    }

    /// `aICH_hover`, also run for rest.
    fn hover(&mut self, world: &World, field: &impl Field, rand: &mut impl FnMut(f32) -> f32) {
        if self.action == Action::Hover {
            self.flap();
        }
        if self.check_patience(world) {
            self.setup(self.scared_action(), Some(world), field, rand);
            return;
        }
        self.timer -= 1.0;
        if self.timer <= 0.0 {
            let next = if self.action == Action::Rest {
                Action::Fly
            } else {
                Action::Rest
            };
            self.setup(next, Some(world), field, rand);
        }
    }
}

/// `mFI_CheckFGNpcOn`: items a villager (or a butterfly's target) can be on.
fn npc_on(item: u16) -> bool {
    const SAPLINGS: [u16; 13] = [
        0x800, 0x805, 0x80D, 0x815, 0x81D, 0x825, 0x82D, 0x832, 0x837, 0x84F, 0x854, 0x85D, 0x863,
    ];
    match item >> 12 {
        _ if item == 0 => true,
        0 => {
            SAPLINGS.contains(&item)
                || (0x83C..=FLOWER_TULIP2).contains(&item)
                || (0x8..=0xA).contains(&item)
        }
        1..=4 | 8 => true,
        _ => false,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Flat grass, acres of 16 units; unit (5, 5) of acre (0, 0) has a
    /// white pansy.
    pub(crate) struct Flat;
    impl Field for Flat {
        fn ground_y(&self, _: f32, _: f32) -> f32 {
            0.0
        }
        fn water_y(&self, _: f32, _: f32) -> Option<f32> {
            None
        }
        fn unit_of(&self, x: f32, z: f32) -> Option<[i32; 4]> {
            let (ux, uz) = ((x / UNIT).floor() as i32, (z / UNIT).floor() as i32);
            Some([
                ux.div_euclid(16),
                uz.div_euclid(16),
                ux.rem_euclid(16),
                uz.rem_euclid(16),
            ])
        }
        fn unit_center(&self, u: [i32; 4]) -> [f32; 2] {
            [
                ((u[0] * 16 + u[2]) as f32 + 0.5) * UNIT,
                ((u[1] * 16 + u[3]) as f32 + 0.5) * UNIT,
            ]
        }
        fn item(&self, u: [i32; 4]) -> Option<u16> {
            Some(if u == [0, 0, 5, 5] {
                FLOWER_PANSIES0
            } else {
                0
            })
        }
        fn blocked(&self, _: [f32; 3], _: [f32; 3]) -> bool {
            false
        }
        fn attribute(&self, _: f32, _: f32) -> Option<u8> {
            Some(0)
        }
    }

    pub(crate) fn world(movers: &[Mover]) -> World<'_> {
        World {
            player: movers.first().map_or([1e6; 3], |m| m.pos),
            player_facing: 0,
            movers,
            net_stop: None,
            scoop: None,
            tree_shaken: None,
            axe_hit: None,
            raining: false,
            busy: false,
        }
    }

    #[test]
    fn butterfly_circles_its_unit_and_stays_in_the_acre() {
        let mut r = |n: f32| n * 0.5;
        let mut b = Insect::butterfly(COMMON_BUTTERFLY, [300.0, 300.0], &Flat, &mut r).unwrap();
        for _ in 0..600 {
            assert_eq!(b.step(&world(&[]), &Flat, &mut r), Event::None);
        }
        assert_eq!(b.action, Action::Fly);
        let u = Flat.unit_of(b.pos[0], b.pos[2]).unwrap();
        assert!((1..=14).contains(&u[2]) && (1..=14).contains(&u[3]));
        assert!(b.pos[1] > 0.0, "keeps off the ground");
    }

    #[test]
    fn over_a_white_pansy_it_lands_and_rests() {
        let mut r = |n: f32| n * 0.4; // stay put: same unit
        let c = Flat.unit_center([0, 0, 5, 5]);
        let mut b = Insect::butterfly(COMMON_BUTTERFLY, c, &Flat, &mut r).unwrap();
        let mut seen = Vec::new();
        for _ in 0..900 {
            b.step(&world(&[]), &Flat, &mut r);
            if seen.last() != Some(&b.action) {
                seen.push(b.action);
            }
        }
        assert!(seen.contains(&Action::Landing), "{seen:?}");
        assert!(seen.contains(&Action::Rest), "{seen:?}");
    }

    #[test]
    fn a_running_player_scares_a_tiger_butterfly_away() {
        let mut r = |n: f32| n * 0.5;
        let mut b = Insect::butterfly(TIGER_BUTTERFLY, [300.0, 300.0], &Flat, &mut r).unwrap();
        let mut acted = Vec::new();
        for _ in 0..120 {
            let p = Mover {
                pos: [b.pos[0] + 30.0, 0.0, b.pos[2]],
                moved: 3.0,
            };
            b.step(&world(&[p]), &Flat, &mut r);
            acted.push(b.action);
        }
        assert!(acted.contains(&Action::Avoid));
    }

    #[test]
    fn flying_off_fades_then_goes() {
        let mut r = |n: f32| n * 0.5;
        let mut b = Insect::butterfly(COMMON_BUTTERFLY, [300.0, 300.0], &Flat, &mut r).unwrap();
        b.setup(Action::LetEscape, None, &Flat, &mut r);
        let ticks = (0..200).position(|_| b.step(&world(&[]), &Flat, &mut r) == Event::Gone);
        assert!(matches!(ticks, Some(70..=90)), "{ticks:?}");
    }
}
