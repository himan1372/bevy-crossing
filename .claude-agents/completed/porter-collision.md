# Porter collision

Philip reports on 2026-10-30 that Porter can be walked through. Add his body to
the existing outdoor movement collision, using his actual entity position and
source body size. Preserve the scripted arrival and open platform/stair route.
Verify walking into him from both sides and walking around him in an isolated
preview. Protect the real save; no tests without request; sequential Cargo gates.

Cause: the arrival spawns Porter's render/animation marker, but player_movement
only includes shop/statue/Copper blockers and has no Porter collider.

## Source and fix

`ac_npc_station_master.c::aSTM_actor_ct` uses the ordinary NPC collision setup;
`ac_npc_ct.c_inc` sets its pipe radius from npc_draw_data_tbl.col_radius.
Read-only disc lookup of entry 252 confirms radius 20, height 45. The player
standing pipe in m_player_common.c_inc is radius 20. Combined planar clearance
is therefore 40 game units (one world unit).

The Porter marker now supplies a circular blocker queried at its entity's live
Transform by outdoor player movement. It does not retain a separate position
resource that can become stale. Queries are explicitly disjoint from Player.
Movement from an overlap may increase the separation but cannot penetrate
farther, preserving escape from a scripted/legacy overlap. Arrival movement
remains scripted and its accepted landing is unchanged.

The blocker checks the closest point on the whole movement segment, not just
the destination. This prevents long-frame running steps from tunneling through
his circle. For an overlapping start, the radial dot product rejects any path
that initially deepens the overlap, including one that ends on the far side.
Focused review confirmed the final swept check; no further issues found.

check/clippy/fmt and capture/default/release builds passed. No tests added or run.

## Runtime evidence

`%TEMP%/bevy-porter-collision-20261030`: arrival completes unchanged, ending
at (-0.5,5,-42.6). East approach stops 1.0009 units from Porter; continued held
input stays blocked. Synthetic west approach also stops at distance 1.0009.
An explicit overlapping setup at distance 0.4 escapes outward to 2.4797. Walking
around his right shoulder and down the stairs reaches (2.4493,4,-36.5811).
Screenshots and exact transforms are retained in the Temp folder, including
checks.json. No new ERROR; the existing iam_sugi_kpool startup warning remains.

Capture/default/release builds and bounded runtime checks pass (release 3m45s).
The automated preview is closed; the user's fresh interactive replay is separate.
No tests added or run. The exact source radii were read without exporting assets.
