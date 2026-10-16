# Retail mail UI

2026-10-16, codex. Philip accepts UI/paper appearance; action/camera corrections
below await his follow-up. Overall mail fidelity remains open.

Second UI task, after retail-pockets-ui primitives.

## Scope

Trace retail letter list, stationery, reading and existing letter actions.
Reuse pockets/font/window primitives and existing mail storage/logic; identify
missing gameplay actions explicitly instead of implying visual work adds them.

Relevant files: app/src/mail.rs, mailbox.rs, pockets.rs, msg_window.rs, input.rs;
original mail/inventory code identified during tracing.

## Acceptance and boundaries

Philip compares the letter list and reading view with retail, including
stationery and available actions. Preserve unread/read behavior, safe navigation
and mailbox/door input separation; do not modify real saved letters during review.

Load original assets at runtime from Philip's own disc; never commit extracted
bytes. Keep the no-disc placeholder usable. Use source and existing translations,
not guessed layouts or gameplay rules. Claim before implementation. Run suitable
compile/lint/format/build checks for code changes; no tests unless Philip requests
them. Record implementation evidence separately from Philip's actual acceptance.
M4 remains open; title and rename remain deferred.

## Implementation

- [x] Runtime unread/read envelope icons in all ten pockets letter positions.
- [x] Right from the item grid enters letters at the same Y (source row+2);
  left returns to items. Mailbox access selects within the 2x5 letter grid.
- [x] Enter opens Read/Grab/Throw Away/Quit; Esc/Backspace returns to the opening context and preserves
  the existing read-on-return behavior. Empty slots remain safe to select.
- [x] Original stationery models, ruling, palette and per-paper text colors.
  The source table has64 populated papers; reserved/null entries use paper1.
- [x] Disc font, source header/body/footer positions, 192px wrapping/six rows.
  Longer port letters paginate with Left/Right or Enter; text is retained.
- [x] Existing N-to-compose from mailbox: typing, recipient arrows, Backspace,
  Enter-to-send, Esc-to-cancel, existing scorer/replies/capacity retained.
- [x] Optional defaulted paper index in saved Letter; old saves remain readable.
- [x] No-disc text panel stays usable. Pause hides the disc view through focus.
- [x] Philip: first UI and paper appearance look fine.
- [ ] Philip: corrected camera, compact action menu, moving/discard acceptance.

app/src/mail_disc.rs renders cached stationery with pockets_disc.rs Paint/font
primitives. mail.rs keeps storage, delivery, scoring and input; pockets.rs adds
letter-grid navigation. quest.rs supplies the default paper for its existing
letter producer. No extracted data is embedded or committed.

Source references: m_submenu_ovl.c::mSM_draw_mail (mtegami/mtegami2 envelope
palette/texture pairs); m_tag_ovl.c mail columns/rows and inventory transition;
m_board_ovl.c::mBD_set_frame_dl, mBD_set_character, mBD_strLineCheck and the
paper_disp_model/paper_disp_sen_model/letter_color tables. Tables/pointers and
graphics are resolved from the user's disc. No translated board renderer exists
in logic; existing mail/scoring behavior remains reused.

## Evidence and remaining scope

Disposable Temp town with ten synthetic letters: envelope grid, read/unread
icons, reading from pockets and return, physical mailbox E (without entering
house), source papers1/2/64, reserved index64 fallback, and page3 showing the
letter's final text visually inspected. Typed a complete draft, changed recipient
from Bob to Kabuki, sent through the existing scorer; Temp save contains the
exact draft, paper0, ten received letters,15 pocket items,12345bells and condition1.
Read flags changed only for viewed/returned letters. Separate explicit missing-
disc startup: physical mailbox open, read, back and close inspected. Screenshots
and synthetic saves stay in Temp. User save hash unchanged, settings untouched.

Workspace check/clippy (all targets, warnings denied), fmt, capture/default
builds pass. No tests added or run. Preview processes closed at handoff.

This uses the port's existing single ten-letter mailbox pool for the pockets
letter grid; separate received-mail transfer/transit and carried-mail storage
are not implemented by visual work. Subjects, page hints and PC keyboard
composition are port adaptations. Original writing/reply text remains the port's
writing. Full retail tag popups/keyboard, stationery selection/purchase, paper
effects/animation, gifts/attachments and sent-mail browsing
remain follow-ups. No unavailable action is presented as working. M4 remains
open; final retail visual/interaction acceptance belongs to Philip.

## Camera regression, letter actions and compact menu correction

Philip reports player movement separating from camera and requests letter move/
discard. The added menu preview cameras made camera_follow's broad With<Camera>
query match multiple cameras, so single_mut returned early. Fixed with a
WorldCamera marker and camera follow ordered after the movement set. Existing
world follow math is unchanged; preview/overlay cameras are excluded by identity.

Source mTG_def_rmail now supplies Read/Grab/Throw Away/Quit behavior in both
pockets and mailbox contexts. Grab carries the envelope/shadow with the closing
hand, swaps occupied slots or moves to empty slots; Cancel never changes storage.
Throw Away uses the source "Throw it out?" confirmation, initially No. Added a
defaulted optional Letter.slot field so old saves still load and empty positions
persist instead of compacting after moves/discards. Complete letter content,
sender, paper and read status stay together. New mail fills a free slot.

Philip additionally reports the new drop-down too big. Letter menus now fit the
actual text bounds plus padding, at the source 0.875 text scale/14px row spacing.
Choice-body bounds are cached during load before Bevy extracts mesh data; an
initial review build tried to read extracted attributes and panicked, caught and
fixed before publishing. The shared item menu retains its previous layout.

Final disposable review: camera/player movement deltas match (~3.65 units in
the first check; a second check after menu use also follows correctly), while
both preview camera transforms remain fixed. Compact menu/confirmation inspected.
Swapped letters0/1, canceled discard with No, confirmed discard of Neighbor0,
moved Neighbor1 into the vacated slot1 leaving slot0 empty, canceled another
move in mailbox and pockets. Temp F5/F9 preserves all nine remaining letters,
their slot/paper/read fields,15 items,12345 bells and condition1; reading after
load shows the correct Neighbor1 stationery/body. Explicit no-disc fallback
also swaps, shows the confirmation, cancels and discards. User data untouched.
Compile/lint/format/capture/default build gates pass; no tests added/run. Philip
has not yet accepted the corrected menu size, movement or action behavior.
