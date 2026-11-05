# The island: Kapp'n's boat, the voyage and the walkable island (virtual GBA)

Decision (Philip, via the question): the island works with a built-in default island instead of a GBA (the decomp's PC build
already skips the GBA chain in `aSEN_pl_ride_on_check2_talk_proc`).

`app/src/kappn.rs`: Kapp'n (model 238 + 114) and a box boat at `aBT_chk_point[0]` (world (29.5, 42.5)); talking to him: 0x3073
(Yes / No), No 0x3074, Yes 0x30BE the first time (the island is named after the town) / 0x3076 later; the voyage along the
eight checkpoints at 0.625 units a frame with the player aboard; departure 0x30BF (0x3084 home), the shanty first part
(`boat::first_part_msg`, never the last song of that direction), the dialogue (`dialogue_msg`), the second part, the wharf
timer, the player stepping out, the arrival line 0x307A (0x308B home). On the island the same boat waits at the last
checkpoint and his question is 0x3080 (Yes 0x3083 / No 0x3081).
`app/src/town.rs`: `Town::on_island` (the blocks (4,8) and (5,8): x 8..40, z 64..80): `can_stand` / `can_step` follow the disc
terrain there, since the plan has no cells outside the 5x6 acres, so the island can be walked.

Run in game (Temp save, disc, 1 Jan 2027): dock talk 0x3073 -> 0x30BE -> 0x30BF -> shanty (0x3091 / 0x3094 ...) -> dialogue
-> second part -> arrival 0x307A at the island wharf; walked the island shore (stumps, snow); on the wharf 0x3080 -> 0x3083
-> 0x3084 -> songs -> 0x308B at the home dock. clippy / fmt clean.
Unverified: No at either question (0x3074 / 0x3081), the song memory across trips, the player's sex patterns (girl: 0x3163 ...),
walking off the island's edges (the bounds are the two blocks), saving while on the island (not handled).
Not built: the boat's model (a box), the oar animation and songs' music, rain and night variants (0x30B9 / 0x308C), the island
naming screen, the island's items / resident / house, the lighthouse and its switch (so Tortimer's quest still cannot finish).
