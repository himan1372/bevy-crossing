# Mail QA: the paths inventory-mail-polish did not drive

2026-11-02, claude. Drive in a Temp copy (synthetic save): Rewrite cancel / commit on a stored letter, long legacy letter paging past page 1,
special tag arrow variants and right-edge flip, Museum recipient colours, Write refusal. Fix what the drive shows; Philip acceptance stays open.

## Result 2026-11-02 (claude)
Driven in `Temp/bevy-mailqa-20261102` (capture exe copy, synthetic save from the earlier mail fixture, 3 letters). Checked: F4 commit is now the native
edit-end window (mail-edit-end-check, so that gap is closed); a 300-char letter reads as 2/2 pages and a further Enter stays on 2/2; the letter tags
(slot 0 and slot 1, both at the right edge) flip to the left of the cursor with the pointer arrow and the Read / Grab / Throw Away / Quit menu below.
Not driven: Rewrite cancel/commit (the menu for received letters has no Rewrite), arrow variants 2 and 3, Museum recipient colours, Write refusal.
No code changed. Philip acceptance open.
