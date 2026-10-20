# Finish the museum

Philip 2026-10-20: "start with copper and booker then finish the museum".
After the first museum stage (e9a36847): fish tanks (ac_museum_fish) and
insect cases (ac_museum_insect) showing donated animals, unidentified
fossils appraised by mail, Blathers' after-talk (0x2F50 choice 2),
completion letter, his sleep animation.

## Progress (claude, 2026-10-20) - stage 1 committed

- Blathers' talk follows aCR's question flow: 0x2F62 before the pockets,
  "anything else?" (0x2F66/0x2F6F/0x2F73), after-talk by count
  (0x2F58-0x2F5B), remarks per insect/fish (0x3A56..), fossil skeleton
  complete (0x2F78+kind / 0x2F84), category complete (0x2F85/75/88/8A),
  all complete (0x2F76/0x2F77). Item name in ITEM_STR0 (was FREE0: bug).
- Doze: WAIT -> SLEEP_WAIT 360 frames -> SLEEP (wait_nemu1) by day;
  greeting index fixed (SLEEP_WAIT = plain welcome, SLEEP = wake-up).
- messages.rs TextBank::talk_asking (which message asks the question).
Next (still claimed): fish tanks (ac_museum_fish: tanks at (220,40,220),
(420,40,220), (220,40,460), (420,40,460), big tank (320,40,20) scale
0.0625; obj_suisou1_model scale 0.01; fish cKF_bs_r_act_mus_* +
cKF_ba_r_act_mus_*, mfish_group_tbl), insects (ac_museum_insect,
act_m_mu_* display lists, roaming), Blathers solid.
Blocked on real mail: fossil appraisal (mMsm_SendMuseumMail /
mMsm_DepositFossil), completion letter (mail 0x22F + FTR_NOG_MUSEUM),
info letter (mail 189).

## Result (claude, 2026-10-20) - stage 2
- app/src/disc_rig.rs: generic skinned cKF rig by name (fan pattern).
- app/src/museum_fish.rs: tanks (obj_suisou1 x4, obj_museum5 big),
  solid from model bounds; donated fish (act_mus_*) swimming per tank.
- app/src/museum_insect.rs: donated insects (act_m_mu_* frames) at kind
  spots; flyers drift between flowers, dragonflies/firefly hover, pond
  skater glides.
- Insect room: rom_museum4_ue_model drawn; IndoorRoom::build_with takes
  segments (disc_town::ground_segments) for the garden's grass/earth;
  raised garden units solid. Blathers solid.
Verified in Temp preview (real save unchanged).
Left: kusa plants, frog lily pad, jellyfish (no skeleton), firefly glow
and per-insect behaviour, water surface (obj_museum4_water_modelT);
mail-based features blocked on real mail.
