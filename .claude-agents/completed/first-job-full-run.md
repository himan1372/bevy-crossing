# The whole part-time job in one run

Philip: "go" (2026-10-28): drive the job from the first greeting to HRATALK the next day in one game process.

## Result (2026-10-28, claude)
Driven in Temp (capture build, disc, tier-0 synthetic save, game clock 15:00, a savegame with the job flag on and the player in a shirt that is not the uniform), with F5/F9 between stages. Message numbers are the shop's
`Nook message` log lines; "real" steps were played through the game's own input, "stand-in" steps edited the saved state for something not built or too long to play:
1. Arrival (real): 0x07EE, 0x07F1; job 1 (change clothes) running, uniform in slot 1.
2. Wearing the uniform in the pockets (real): `progress 2 -> 0`; talking to Nook: 0x07F4, 0x07F6; job 2 (plant) running, 10 plant items.
3. Planting (stand-in: pockets emptied): `progress 2 -> 0`; Nook 0x07F9; the chain went to introductions (not everyone met), job 10 running.
4. Introductions (real): Tortimer talked to (trophy set), all six villagers greeted (three retries for the one on the cliff): `progress 2 -> 0`; Nook 0x0828 then 0x07FB; job 3 running, recipient Ace, furniture 0x1324.
5. Furniture (real): delivered to Ace (progress 0); Nook 0x0801; job 4 running (letter), recipient Anchovy.
6. Letter (stand-in: Anchovy added to `post.sent_to`): `progress 2 -> 0`; Nook 0x080B then 0x0827; the open job running.
7. Open quest (real): Freya's offer accepted: `progress 2 -> 0`; Nook 0x0830 then 0x080C; carpet job running, recipient Bluebear.
8. Carpet (real): delivered; Nook 0x0812 then 0x0813; axe job running, recipient Anchovy (the letter's).
9. Axe (real): delivered; Nook 0x0818 (the end of the axe job and the notice job's start share this message in the source); notice job running.
10. Notice (real): the board opened, "hi" written and posted (`progress 2 -> 0 (notice posted)`); Nook 0x081A; the job ended: events `[false, true, false]` (HRAWAIT) and the errand dropped.
11. The next day (stand-in: `events_day` reset so the daily renewal runs): HRAWAIT became HRATALK (`[false, false, true]`) and entering the shop gave Nook's 0x082A greeting.
No WARN/ERROR from the game (one unrelated gamepad-mapping WARN from an earlier launch).

## Bugs the run found and fixed
- The part-time job's villager greeting did not count as meeting the villager (see `completed/first-job-introductions.md`).
- The app's own quest offers kept appearing while the job ran (a Freya offer panel blocked saving mid-job): `quest.rs::job_blocks_quests` now stops new offers and the offer talk while the job runs, except during the open-quest job (`aQMgr_select_talk`).
- A new game's shirt comes from the savegame present when the game starts (F9 does not reset it): the harness must write the save before launch.
Gates: fmt, clippy -D warnings (workspace, all targets) after the quest change. No kernel change.
Not run: the second letter and second axe jobs (only the "recipient moved away" redirects), the town-map hand-over, the intro wipe, the wrong-clothes scolding (0x0832), the refusal paths with full pockets, and the day boundary itself (the renewal was forced).
