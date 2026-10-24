# Raffle ball double scale

Found during catalog-raffle-verification on2026-10-24 at372e29bf. Not fixed.

RaffleBall changes Hidden->Inherited and its translation follows the roll,
but the ball is not discernible in inspected screenshots. load_ball_frames
in app/src/raffle.rs builds via disc_town::build_model_full, which already
applies OBJECT_SCALE(0.00025) to mesh vertices. present applies GOODS_SCALE
(0.00025) to its parent again. Live child Aabb half width0.0625 yields only
0.00003125 world-unit diameter after the extra parent scale (4000x too small).
Original ac_garagara_ball.c_inc aGRBL_draw applies0.01 once in game units.

Next proposed fix: use one model-to-world scale, preserving per-frame ball
texture/position/billboarding; inspect the visible ball leaving the drum and
rolling to rest in a synthetic Temp preview. Check ticket transactions remain
unchanged. Read texture/QA/Rust/Bevy skills first; no tests unless requested.

Evidence is in the completed/catalog-raffle-verification.md task. Parent
nook-tickets-catalog remains open and claude-owned. This note does not grant
permission to take over that claim or mark parent acceptance complete.
