# Part-time job gaps

Philip: "do job gaps and open from pause" (2026-10-28). From the handoff's known gaps: the 0x17B6 window colour, the fanfare length, then (as far as the source allows) the second letter / second axe jobs and the moved-away redirects.

## Slice 1 (2026-10-28, claude): window colour and fanfare length
- `DialogueState.window_color` (`mDemo_Set_talk_window_color`) + `MsgAssets.window_cheer`: the job-end cheer 0x17B6 is drawn in the source's green (185,245,80).
- Fanfare length: BGM_INTRO_CHORES_COMPLETE (sequence 232) never ends by itself (45 s with the message unread, no stop) - the source withdraws it when the message is read, so the 14 s guess is replaced by "until read" (cap one hour).
Verified (Temp capture copy, synthetic save with job 6 done, disc): the end talk 0x081A, the cheer 0x17B6 in a green window with no name plate, fanfare held past 45 s, withdrawn on reading (town music returns). Gates: fmt, clippy -D warnings, capture build. Real save unchanged.

## Left open (assessed, not started)
- SendLetter2 / DeliverAxe2: the guide and talk code already handle both kinds (`guide.rs`, `first_job_talk.rs`); in `first_job.rs::next` nothing leads to them in an ordinary run, they belong to the "recipient moved away" redirects, which need a villager leaving town mid-job (not modelled).
- Hand-over visuals for job items, the axe delivery's reply-letter display, the map hand-over animation and the real scene wipe: separate, larger pieces; each needs its own source audit.
