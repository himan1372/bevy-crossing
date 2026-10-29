# First job: "recipient moved away" redirects, the second letter / axe jobs, the paper choice

Philip: "job redirects" (2026-10-28). Source: `aNRG2_setup_j3/4/5/6_cont` (`mQst_CheckRemoveTarget`), `dt_tbl` of
`ac_npc_rcn_guide2_talk.c_inc` (0x07FF / 0x0810 `demo2_start_wait`, 0x0816 `demo3_start_wait`, 0x0808
`decide_new_target`, 0x0804 `give_new_paper`, 0x0807 -> 0x080A `check_more_paper`), `m_npc.c` (a delivered
letter sets progress 3 during the letter jobs).

Plan: kernel `Job::moved_away`, `FirstJob::recipient_gone` / `letter_sent`; `guide.rs` talks and effects
(take the item back with a player -> Nook hand-over, restart the job with a new pick, or for the axe the second
letter; wrong-recipient letter and the "more paper" choice); `GameText::talk_until` so a redirect message
can be rendered before the new recipient's name is put in; second letter / second axe handled like the first.

## Done (2026-10-28)
- Kernel (`logic/src/first_job.rs`): `Job::moved_away` (message + replacing job), `moved_away_next`, `given_back`; `FirstJob::recipient_gone`, `letter_sent` (progress 3); two more own tests (10 pass).
- `GameText::talk_until` renders a redirect message on its own so the new recipient's name can go in before the message that follows (`guide::single`).
- `guide.rs`: the recipient test (`Residents::with_gone` names the missing villager from the disc table); furniture/carpet: the order (NPC0, 9) takes the item from the pockets and plays the player -> Nook hand-over (`dialogue.held`), then the same job restarts with a new villager and item; axe/axe2: the axe is handed back and the second letter starts; letter/letter2: 0x0808, new target, 0x0809, then 0x080A asks about more paper (`paper_answer`); progress 3 (a letter given to someone else, `Post.letters_given`) says 0x0804 and gives paper (0x0805, or 0x07FA with full pockets); 0x0807's own question about more paper is answered too. Second letter / second axe are handled like the first.
- Driven (Temp capture copy, disc, synthetic save, `GONE` = villager 200 not in town): furniture (0x07FF, hand-back, new recipient and item, 0x0800, new hand-over of the fan), carpet not separately, axe (hand-back, second letter with paper), letter (0x0808, 0x0809, 0x080A yes -> 0x0805 second paper), wrong recipient (progress 3 -> 0x0804, 0x0805, paper), paper question (yes). Real save hash unchanged.
- Villagers do not move out in this port, so the redirects only fire from a save with a recipient who is not a resident; a move-out would use them.
- Not done: the "no" answer and the full-pockets paper path were not driven; carpet redirect shares the furniture code; an equipped axe is not searched when the quest axe is not in the pockets (`demo3_start_wait` clears the equipment); the source clears pocket 0 when the quest item is missing (kept).
