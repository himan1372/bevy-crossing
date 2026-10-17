# Rustimal migration inventory audit

2026-10-16, Codex. Read-only Git-tree comparison; no module migration performed.

Source: `himan1372/rustimal-crossing` commit `122867f762122eb266f04e319af0f0c31d54f476` (`rust/src/`).
Target snapshot: `himan1372/bevy-crossing` commit `a2005e7a435484999ced5e5bc1dcaf476eb48b3b` (`logic/src/`), before verification corrections.

Counts use full relative paths recursively, excluding only the crate-root `lib.rs` on each side. This is a file inventory, not proof of module registration, production suitability, or behavioral equivalence. Disc/app crates are outside this count.

- Source modules: 194
- Target modules: 117
- Shared paths: 90 (2 exact blob matches, 88 different)
- Source-only paths: 104
- Bevy-only paths: 27

The former 113-new/81-overlap/10-Bevy-only counts mixed directory levels. All nine `graphics/*.rs` paths already exist on both sides. Preserve Bevy graphics/runtime behavior and the 27 Bevy-only files. `test_c_stubs.rs` is source-only but is a C test harness, not a production gameplay migration target. Compare changes per module and against C; neither file age nor byte differences establishes which behavior should win.

Furniture was already present before the pilot; the pilot updated it rather than adding a missing module. Its original imported baseline matches the pinned source apart from a module preamble and removal of ABI export attributes/calling conventions. See the migration task note for correction and source-gap evidence.

The later authorized furniture correction changes native loader/storage APIs,
adds per-layer occupancy and fixes banked item IDs. Preserve these verified
adaptations when comparing future source revisions; this inventory's hashes and
counts remain the original pinned snapshot, not a new claim of file equivalence.

| Relative module path | Inventory status |
| --- | --- |
| `action_audio.rs` | Bevy only; preserve |
| `actor_shadow.rs` | Source only; review suitability/dependencies |
| `address_book.rs` | Source only; review suitability/dependencies |
| `albumin.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `albumin_collision_data.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `albumin_geometry.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `ambience.rs` | Bevy only; preserve |
| `aprilfool.rs` | Source only; review suitability/dependencies |
| `aram.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `attr_walls.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `attribute_action.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `audio.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `audio_mix.rs` | Bevy only; preserve |
| `balloon.rs` | Source only; review suitability/dependencies |
| `bank.rs` | Source only; review suitability/dependencies |
| `bee_ant.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `behavior.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `bg_check.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `birth_control.rs` | Source only; review suitability/dependencies |
| `birthday_ovl.rs` | Source only; review suitability/dependencies |
| `blanca.rs` | Source only; review suitability/dependencies |
| `boat.rs` | Source only; review suitability/dependencies |
| `bridge_acre.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `buildings.rs` | Source only; review suitability/dependencies |
| `buried_items.rs` | Shared; exact blob match |
| `calendar.rs` | Source only; review suitability/dependencies |
| `camera.rs` | Source only; review suitability/dependencies |
| `catalog.rs` | Source only; review suitability/dependencies |
| `cherry_blossom.rs` | Source only; review suitability/dependencies |
| `choice.rs` | Source only; review suitability/dependencies |
| `cockroach.rs` | Source only; review suitability/dependencies |
| `collision.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `collision_temporal.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `column_sweep.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `columns.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `controller.rs` | Source only; review suitability/dependencies |
| `countdown.rs` | Source only; review suitability/dependencies |
| `decal_circles.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `demo_actors.rs` | Source only; review suitability/dependencies |
| `design.rs` | Source only; review suitability/dependencies |
| `dialogue_topics.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `diary.rs` | Source only; review suitability/dependencies |
| `dvd.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `ecology.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `endpoint_circle.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `engine_tables.rs` | Source only; review suitability/dependencies |
| `event_day.rs` | Source only; review suitability/dependencies |
| `event_map_npc.rs` | Source only; review suitability/dependencies |
| `event_visitors.rs` | Source only; review suitability/dependencies |
| `famicom.rs` | Source only; review suitability/dependencies |
| `fg_data.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `field_assessment.rs` | Source only; review suitability/dependencies |
| `field_effects.rs` | Source only; review suitability/dependencies |
| `field_gen.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `field_items.rs` | Bevy only; preserve |
| `fireworks.rs` | Source only; review suitability/dependencies |
| `fish_actor.rs` | Source only; review suitability/dependencies |
| `fish_tables.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `fishing_tourney.rs` | Source only; review suitability/dependencies |
| `fishrecord.rs` | Source only; review suitability/dependencies |
| `footsteps.rs` | Bevy only; preserve |
| `force_call.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `frame_loops.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `furniture.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `game_math.rs` | Source only; review suitability/dependencies |
| `game_math_tables.rs` | Source only; review suitability/dependencies |
| `game_string.rs` | Source only; review suitability/dependencies |
| `game_time.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `gbi_runtime.rs` | Shared; exact blob match |
| `go_home_npc.rs` | Source only; review suitability/dependencies |
| `graphics.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `graphics/combine.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `graphics/command.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `graphics/frame.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `graphics/gx_backend.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `graphics/interpreter.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `graphics/segments.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `graphics/state.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `graphics/texture.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `graphics/vertex.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `groundhog.rs` | Source only; review suitability/dependencies |
| `gulliver.rs` | Source only; review suitability/dependencies |
| `gyo_ai.rs` | Bevy only; preserve |
| `halloween.rs` | Source only; review suitability/dependencies |
| `hand_over_item.rs` | Source only; review suitability/dependencies |
| `haniwa.rs` | Source only; review suitability/dependencies |
| `harvest.rs` | Source only; review suitability/dependencies |
| `hatsumode.rs` | Source only; review suitability/dependencies |
| `holiday_scenery.rs` | Source only; review suitability/dependencies |
| `house.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `house_board.rs` | Source only; review suitability/dependencies |
| `house_interior.rs` | Source only; review suitability/dependencies |
| `house_scene.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `hud.rs` | Source only; review suitability/dependencies |
| `ins_ai/amenbo.rs` | Bevy only; preserve |
| `ins_ai/batta.rs` | Bevy only; preserve |
| `ins_ai/dango.rs` | Bevy only; preserve |
| `ins_ai/goki.rs` | Bevy only; preserve |
| `ins_ai/hitodama.rs` | Bevy only; preserve |
| `ins_ai/hotaru.rs` | Bevy only; preserve |
| `ins_ai/ka.rs` | Bevy only; preserve |
| `ins_ai/kabuto.rs` | Bevy only; preserve |
| `ins_ai/kera.rs` | Bevy only; preserve |
| `ins_ai/mino.rs` | Bevy only; preserve |
| `ins_ai/mod.rs` | Bevy only; preserve |
| `ins_ai/semi.rs` | Bevy only; preserve |
| `ins_ai/tentou.rs` | Bevy only; preserve |
| `ins_ai/tonbo.rs` | Bevy only; preserve |
| `ins_set.rs` | Bevy only; preserve |
| `insect_actor.rs` | Source only; review suitability/dependencies |
| `insect_tables.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `interaction.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `inventory.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `inventory_ui.rs` | Source only; review suitability/dependencies |
| `island.rs` | Source only; review suitability/dependencies |
| `item_prefs.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `kamakura.rs` | Source only; review suitability/dependencies |
| `kankyo_env.rs` | Source only; review suitability/dependencies |
| `kk_credits.rs` | Source only; review suitability/dependencies |
| `leaflet.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `letter_score.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `letter_score_tables.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `lights.rs` | Source only; review suitability/dependencies |
| `lost_found.rs` | Source only; review suitability/dependencies |
| `mail.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `majin.rs` | Source only; review suitability/dependencies |
| `mck_key_tables.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `melody.rs` | Source only; review suitability/dependencies |
| `menu_overlays.rs` | Source only; review suitability/dependencies |
| `moon_viewing.rs` | Source only; review suitability/dependencies |
| `mother_mail.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `move_bg.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `movement.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `mtx.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `museum.rs` | Source only; review suitability/dependencies |
| `museum_display.rs` | Source only; review suitability/dependencies |
| `mushroom.rs` | Source only; review suitability/dependencies |
| `na_voice.rs` | Bevy only; preserve |
| `name_editor.rs` | Source only; review suitability/dependencies |
| `nas_bank.rs` | Bevy only; preserve |
| `nas_seq.rs` | Bevy only; preserve |
| `notice_overlay.rs` | Source only; review suitability/dependencies |
| `npc.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `npc_ai.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `npc_event_mail.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `npc_life.rs` | Source only; review suitability/dependencies |
| `npc_reply.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `npc_set_manager.rs` | Source only; review suitability/dependencies |
| `npc_walk.rs` | Source only; review suitability/dependencies |
| `password.rs` | Source only; review suitability/dependencies |
| `pause.rs` | Source only; review suitability/dependencies |
| `placement.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `player_action.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `player_move.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `player_tools.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `police.rs` | Source only; review suitability/dependencies |
| `post_office.rs` | Source only; review suitability/dependencies |
| `profiler.rs` | Source only; review suitability/dependencies |
| `props.rs` | Source only; review suitability/dependencies |
| `quest.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `quest_firstjob.rs` | Source only; review suitability/dependencies |
| `quest_gen.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `radio.rs` | Source only; review suitability/dependencies |
| `random_field.rs` | Bevy only; preserve |
| `random_field_tables.rs` | Bevy only; preserve |
| `repay.rs` | Source only; review suitability/dependencies |
| `request_selector.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `room_scoring.rs` | Source only; review suitability/dependencies |
| `room_scoring_tables.rs` | Source only; review suitability/dependencies |
| `save.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `save_format.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `scene.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `scene_layout.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `scene_music.rs` | Bevy only; preserve |
| `scene_table.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `segment_map.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `shop.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `shop_interior.rs` | Source only; review suitability/dependencies |
| `slate_classify.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `snowman.rs` | Source only; review suitability/dependencies |
| `special_delivery.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `species.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `stalk_market.rs` | Source only; review suitability/dependencies |
| `step3_data.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `submenu.rs` | Source only; review suitability/dependencies |
| `taisou.rs` | Source only; review suitability/dependencies |
| `talk_request.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `talk_topics.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `tamaire.rs` | Source only; review suitability/dependencies |
| `template_select.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `terrain_walls.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `test_c_stubs.rs` | Source only; C test harness, exclude from production |
| `title_demo.rs` | Source only; review suitability/dependencies |
| `tokyoso.rs` | Source only; review suitability/dependencies |
| `tool_resolvers.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `tortimer.rs` | Source only; review suitability/dependencies |
| `totakeke.rs` | Source only; review suitability/dependencies |
| `town_board.rs` | Source only; review suitability/dependencies |
| `town_gen.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `town_map.rs` | Source only; review suitability/dependencies |
| `train.rs` | Source only; review suitability/dependencies |
| `tunahiki.rs` | Source only; review suitability/dependencies |
| `uki.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `vi.rs` | Source only; review suitability/dependencies |
| `view.rs` | Source only; review suitability/dependencies |
| `villager_ai.rs` | Source only; review suitability/dependencies |
| `villager_favor.rs` | Source only; review suitability/dependencies |
| `villager_home.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `villager_mail.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `villager_memory.rs` | Source only; review suitability/dependencies |
| `visitors.rs` | Source only; review suitability/dependencies |
| `wall_hit_dir.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `wall_priority.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `wall_solver.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `weather_actor.rs` | Source only; review suitability/dependencies |
| `weather_season.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `weed_pull.rs` | Source only; review suitability/dependencies |
| `wind.rs` | Source only; review suitability/dependencies |
| `wisp.rs` | Source only; review suitability/dependencies |
| `wpos2attribute.rs` | Shared; compare/merge, preserve Bevy adaptations |
| `yomise.rs` | Source only; review suitability/dependencies |
