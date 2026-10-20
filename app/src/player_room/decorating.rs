//! mTG_cover_proc/mTG_carpet_proc and aMI_ChangeWall/ChangeCarpet.
use super::*;

/// Standard room banks contain 71 patterns each, including four shop patterns.
/// Custom designs use different private-player texture banks and are separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Decoration {
    pub floor: u8,
    pub wall: u8,
}

impl Decoration {
    pub(crate) fn defaults(disc: &GameDisc) -> Option<Self> {
        let table = disc.catalog.get("l_mHm_player_room_default_data")?;
        let bytes = disc.rom.slice(table.source, table.offset, 2)?;
        Some(Self {
            wall: bytes[0],
            floor: bytes[1],
        })
    }

    fn standard(self) -> bool {
        self.floor < 71 && self.wall < 71
    }

    pub(crate) fn available(self, disc: &GameDisc) -> bool {
        self.standard()
            && disc.player_room_floors.len() >= (usize::from(self.floor) + 1) * 0x2020
            && disc.player_room_walls.len() >= (usize::from(self.wall) + 1) * 0x1020
            && disc.catalog.get(&crate::home::room_lists().0).is_some()
    }
}

pub(crate) fn action(item: u16) -> Option<&'static str> {
    match item {
        0x2600..=0x2646 => Some("Spread on floor"),
        0x2700..=0x2746 => Some("Hang on wall"),
        _ => None,
    }
}

pub(super) fn apply(
    room: &mut PlayerRoom,
    inventory: &mut PlayerInventory,
    slot: usize,
    item: u16,
    disc: &GameDisc,
) -> Result<(), &'static str> {
    let mut next = room
        .decoration
        .ok_or("That room pattern\nisn't available yet.")?;
    if !next.standard() {
        return Err("That room pattern\nisn't supported yet.");
    }
    let previous = match item {
        0x2600..=0x2646 => {
            let old = 0x2600 + u16::from(next.floor);
            next.floor = (item - 0x2600) as u8;
            old
        }
        0x2700..=0x2746 => {
            let old = 0x2700 + u16::from(next.wall);
            next.wall = (item - 0x2700) as u8;
            old
        }
        _ => return Err("Choose a wallpaper\nor carpet first."),
    };
    if !next.available(disc) {
        return Err("This pattern isn't on\nthe selected disc.");
    }
    // The original replaces this very slot, so even full pockets can decorate.
    inventory.pockets.pockets[slot] = previous;
    inventory.pockets.set_item_cond(slot, 0);
    room.decoration = Some(next);
    room.decoration_pending = true;
    Ok(())
}

/// Rebuild only the shell, leaving furniture, collision and transient hands alone.
/// IndoorFloor is on the new root, so carpet selection also updates footsteps.
#[allow(clippy::too_many_arguments)]
pub(super) fn render(
    mut commands: Commands,
    mut room: ResMut<PlayerRoom>,
    disc: Option<Res<GameDisc>>,
    pockets: Option<Res<crate::pockets_disc::PocketAssets>>,
    mut focus: ResMut<InputFocus>,
    shells: Query<(Entity, &crate::interior::PlayerRoomShell)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<crate::n64_material::N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (Some(disc), Some(decoration)) = (disc.as_deref(), room.decoration) else {
        return;
    };
    if pockets.as_deref().is_some_and(|p| !p.world_ready()) || !decoration.available(disc) {
        return;
    }
    let floor = crate::home::floor();
    if !shells
        .iter()
        .any(|(_, shell)| shell.0 == decoration && shell.1 == floor)
    {
        if crate::interior::spawn_decorated_player_room(
            &mut commands,
            disc,
            crate::house::INTERIOR_ORIGIN,
            decoration,
            (&mut meshes, &mut materials, &mut images),
        )
        .is_none()
        {
            return;
        }
        for (entity, _) in &shells {
            commands.entity(entity).despawn();
        }
    }
    if std::mem::take(&mut room.decoration_pending) {
        crate::sound::se(0x11B); // aMI_Change2ReservedWall/Floor.
        if *focus == InputFocus::Furniture {
            *focus = InputFocus::World;
        }
    }
}
