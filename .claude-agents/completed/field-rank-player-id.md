# Field assessment rank and personal ID inputs

2026-10-27 claude, Philip: "do id rank and assessment inputs; use the ACreTeam ac-decomp if you need to know where something is".
Follow-up to completed/shop-bargain-sales.md limits (rank 0 and a name/seed hash stand in for `mFAs_GetFieldRank` and `player_ID.player_id`).

- [x] Source audit: mFAs_SetFieldRank/GetFieldRank, dump anchor (mAGrw_SearchDump), player_ID generation, callers.
- [x] Port rustimal-crossing field_assessment.rs into logic/ (item-id based scan, dust flag), verify item classes against the port's ids.
- [x] App: FieldRank resource computed at start/daily/before scheduling; dust_flag saved (serde default).
- [x] Persist a real player id (0xF000 | rand*253), legacy saves get one once.
- [x] Wire consumers: bargain gap, insect hatch (ecology.rs), villager move-in if source-matching; record absent ones.
- [x] Temp QA (fresh town, litter, bargain gap, id persistence); gates; docs/comms; release; commit/push.

## Implemented (2026-10-27, claude)
Source (ACGC-PC-Port == ac-decomp for these files): `m_field_assessment.c` (SetFieldRank/GetFieldRank/GetDustNum), `m_all_grow_ovl.c` (SearchDump, daily SetFieldRank), `m_event.c` init_special_event (player_ID seed, rank gap), `m_private.c` (player_id = 0xF000 | fqrand*253), `m_npc.c` CheckGrowFieldRank.
- `logic/src/field_assessment.rs`: ported from rustimal-crossing (item-id scan over the 30 acres, dust gate, shrine conditions, good-field streak, event gap); its 5 tests (plus town_gen's assessment test) pass.
- `app/src/field_rank.rs`: `FieldRank` resource (the stored rank, never saved in the original) refreshed when the town's items first exist, each game day, after F9 and just before the bargain scheduler rolls its gap. `dust_flag` lives in `FieldItems` and is saved (`SaveData.dust_flag`, serde default, omitted when false). The dump is found by its 0x583B item (the load-time bug that ignores it is not modelled). Good-field streak not kept.
- `save::PlayerIdentity.id` (serde default 0) assigned once by `assign_player_id` as 0xF000 | draw%253 (draw from the town seed and names so a legacy save gets the same id every time; the original draws randomly at creation). `bargain::tick` now seeds from it instead of the name hash, so NEW offers/events differ from before; saved calendars are untouched.
- Consumers wired: bargain event gap (`Calendar.rank` kept as the last rank used), insect hatch (`ecology.rs`, was fixed 6).
Verified (Temp, disc, legacy-style save): fresh town trees 279 (equals the 279 drawn from the disc), 12 perfect / 9 good / 9 bad acres, score 15, rank 5, id 0xF01E, event gap 2 (the minimum for rank 5, as computed); 5 trash items -> rank 0, `dust_flag` in the F5 save, same id after reload; clearing the trash and F9 -> flag cleared, rank 5 again. No WARN/ERROR.
Not wired (recorded): villager move-in (`roster::init_roster` checks before the town exists, so no rank; stays at the rank-3 70%), fish spawn (the app's fish path has no env_rate), balloon odds, shrine talk and golden axe (absent in the port). Sale-hour and offer RNG remain local adapters, not the original global RNG.
