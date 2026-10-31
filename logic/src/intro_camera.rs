//! Outdoor intro views from m_camera2.c. Positions/distances are game units;
//! angles are binary angles measured from the town camera's +Z eye offset.

#[derive(Clone, Copy, Debug)]
pub struct View {
    pub center: [f32; 3],
    pub distance: f32,
    pub pitch: i16,
    pub yaw: i16,
}

impl View {
    pub fn normal(center: [f32; 3]) -> Self {
        Self {
            center,
            distance: 620.0,
            pitch: 8192,
            yaw: 0,
        }
    }

    /// Camera2_Talk_*: midpoint of eye positions, lowered for the message window.
    pub fn talk(player: [f32; 3], npc: [f32; 3], npc_eye: f32, high_cliff: bool) -> Self {
        let delta: [f32; 3] = std::array::from_fn(|i| npc[i] - player[i]);
        let between = delta.iter().map(|n| n * n).sum::<f32>().sqrt();
        let mut center = std::array::from_fn(|i| (player[i] + npc[i]) * 0.5);
        center[1] += (33.0 + npc_eye) * 0.5 - 17.0;
        if between > 0.0 {
            center[1] -= 60.0 / between;
        }
        if high_cliff {
            return Self {
                center,
                distance: 620.0 * 0.75,
                pitch: 8192 - degrees(2.75),
                yaw: 0,
            };
        }
        let angle = delta[0].atan2(delta[2]);
        let binary = angle * 65536.0 / std::f32::consts::TAU;
        let side_on = binary.abs() > 8192.0 && binary.abs() < 24576.0;
        let correction = if side_on {
            0
        } else {
            let add = ((2.0 * angle).cos() * 2730.0) as i16;
            if angle.cos() * angle.sin() >= 0.0 {
                -add
            } else {
                add
            }
        };
        Self {
            center,
            distance: 290.0 + between * 1.46,
            pitch: degrees(-164.114).wrapping_add(i16::MIN).wrapping_add(1440),
            yaw: correction,
        }
    }

    /// Camera2_MoveCenterPosAndSpeed / MoveDistance / MoveDirection, one60Hz tick.
    pub fn step(&mut self, goal: Self) {
        for (value, target) in self.center.iter_mut().zip(goal.center) {
            crate::room_light::add_calc(value, target, 0.133_974_61, 8.75, 0.25);
        }
        crate::room_light::add_calc(&mut self.distance, goal.distance, 0.133_974_61, 8.75, 0.25);
        chase_angle(&mut self.pitch, goal.pitch, 100);
        chase_angle(&mut self.yaw, goal.yaw, 75);
    }

    pub fn eye(self) -> [f32; 3] {
        let radians = std::f32::consts::TAU / 65536.0;
        let pitch = f32::from(self.pitch) * radians;
        let yaw = f32::from(self.yaw) * radians;
        [
            self.center[0] + self.distance * pitch.cos() * yaw.sin(),
            self.center[1] + self.distance * pitch.sin(),
            self.center[2] + self.distance * pitch.cos() * yaw.cos(),
        ]
    }

    pub fn normal_angle(self) -> bool {
        self.pitch == 8192 && self.yaw == 0 && (self.distance - 620.0).abs() < 0.01
    }
}

fn degrees(value: f32) -> i16 {
    (value * (65536.0 / 360.0)) as i16
}

fn chase_angle(value: &mut i16, target: i16, minimum: i16) {
    let delta = target.wrapping_sub(*value);
    let step = ((f32::from(delta) * 0.051_316_68)
        .abs()
        .clamp(f32::from(minimum), 11000.0) as i16)
        .min(delta.saturating_abs());
    *value = value.wrapping_add(if delta < 0 { -step } else { step });
}
