//! Which villagers remember the player (`Anmmem_c`, by name): one meeting,
//! a talk or a letter (`mNpc_SetAnimalPersonalID2Memory`,
//! `mNpc_ForceGetFreeAnimalMemoryIdx` in mNpc_SendMailtoNpc), is enough.
//! Only the address book reads it so far.
use std::collections::BTreeSet;

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use crate::interaction::TalkEvent;
use crate::npc::Villager;

#[derive(Resource, Default, Clone, Debug)]
pub struct VillagerMemory(pub BTreeSet<String>);

impl VillagerMemory {
    pub fn remembers(&self, name: &str) -> bool {
        self.0.contains(name)
    }

    pub fn meet(&mut self, name: &str) {
        if self.0.insert(name.to_string()) {
            info!("{name} remembers the player");
        }
    }
}

fn load(loaded: Res<crate::save::LoadedSave>, mut memory: ResMut<VillagerMemory>) {
    if let Some(save) = loaded.0.as_ref() {
        memory.0 = save.villager_memory.iter().cloned().collect();
    }
}

fn on_talk(
    mut talks: MessageReader<TalkEvent>,
    villagers: Query<&Villager>,
    mut memory: ResMut<VillagerMemory>,
) {
    for talk in talks.read() {
        if let Ok(v) = villagers.get(talk.villager) {
            memory.meet(v.name);
        }
    }
}

pub struct VillagerMemoryPlugin;

impl Plugin for VillagerMemoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VillagerMemory>()
            .add_systems(Startup, load)
            .add_systems(Update, on_talk.in_set(crate::states::GameplaySet));
    }
}
