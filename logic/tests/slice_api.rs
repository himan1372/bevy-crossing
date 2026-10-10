//! The slice modules are reachable from outside the crate (as the Bevy app
//! will use them) and their core calls behave as translated.

use rustimal_logic::{
    behavior, collision, game_time, inventory, item_prefs, npc_ai, player_move, request_selector,
    talk_request, talk_topics,
};

#[test]
fn player_move_angles() {
    assert_eq!(player_move::wrap_deg(370.0), 10.0);
    assert_eq!(player_move::wrap_deg(-190.0), 170.0);
    assert_eq!(player_move::calc_ease(0.0), 0.0);
    assert_eq!(player_move::calc_ease(1.0), 1.0);
}

#[test]
fn inventory_has_15_empty_pockets() {
    let mut inv = inventory::Inventory::default();
    assert_eq!(inv.pockets.len(), inventory::POCKETS_SLOT_COUNT);
    assert_eq!(inventory::POCKETS_SLOT_COUNT, 15);
    assert!(inv.pockets.iter().all(|&i| i == inventory::EMPTY_NO));
    inv.set_item_cond(3, 2);
    assert_eq!(inv.item_cond(3), 2);
    assert_eq!(inv.item_cond(2), 0);
}

#[test]
fn item_prefs_goods_source() {
    assert_eq!(
        item_prefs::goods_source_for_furniture(0),
        item_prefs::GoodsSource::VillagerHouse
    );
    assert_eq!(
        item_prefs::goods_source_for_furniture(7),
        item_prefs::GoodsSource::RandomItem
    );
}

#[test]
fn talk_request_gate() {
    assert!(talk_request::normal_talk_request_gate(true, true));
    assert!(!talk_request::normal_talk_request_gate(true, false));
    assert!(!talk_request::normal_talk_request_gate(false, true));
}

#[test]
fn talk_topics_prob_table() {
    // rng always 0: the shuffles are no-ops and entry 0 (topic 0) is picked.
    let mut zero = |_bound: u32| 0;
    assert_eq!(
        talk_topics::decide_idx_prob_table(&talk_topics::KI_PROB, &mut zero),
        0
    );
    // rng always 99: the last entry belongs to the last weighted topic.
    let mut last = |_bound: u32| 99;
    assert_eq!(
        talk_topics::decide_idx_prob_table(&talk_topics::KI_PROB, &mut last),
        talk_topics::KI_PROB.len() - 1
    );
}

#[test]
fn request_selector_first_eligible() {
    let picked = request_selector::pick_first_eligible(&[0x10, 0x20, 0x30], &[0, 1, 1]).unwrap();
    assert_eq!(picked.pocket_idx, 1);
    assert_eq!(picked.item, 0x20);
    assert!(request_selector::pick_first_eligible(&[0x10], &[0]).is_none());
}

#[test]
fn npc_ai_schedule_override() {
    assert_eq!(npc_ai::global_schedule_override(false, false), None);
    assert_eq!(
        npc_ai::global_schedule_override(false, true),
        Some(npc_ai::sched::FIELD)
    );
}

#[test]
fn behavior_action_table_size() {
    assert_eq!(behavior::BEHAVIOR_ACTION_NUM, 23);
}

#[test]
fn collision_flat_unit_has_no_slope() {
    assert!(collision::slate_dir(collision::CollisionData { raw: 0 }).is_none());
}

#[test]
fn game_time_epoch_round_trip() {
    let epoch = game_time::ticks_to_calendar_time(0);
    assert_eq!((epoch.year, epoch.mon, epoch.mday), (2000, 0, 1));
    assert_eq!(game_time::calendar_time_to_ticks(&epoch), 0);
}
