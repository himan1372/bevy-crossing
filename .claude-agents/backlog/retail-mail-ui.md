# Retail mail UI

2026-10-16, codex. First implementation; Philip comparison pending.

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
- [x] Enter reads; Esc/Backspace returns to the opening context and preserves
  the existing read-on-return behavior. Empty slots remain safe to select.
- [x] Original stationery models, ruling, palette and per-paper text colors.
  The source table has64 populated papers; reserved/null entries use paper1.
- [x] Disc font, source header/body/footer positions, 192px wrapping/six rows.
  Longer port letters paginate with Left/Right or Enter; text is retained.
- [x] Existing N-to-compose from mailbox: typing, recipient arrows, Backspace,
  Enter-to-send, Esc-to-cancel, existing scorer/replies/capacity retained.
- [x] Optional defaulted paper index in saved Letter; old saves remain readable.
- [x] No-disc text panel stays usable. Pause hides the disc view through focus.
- [ ] Philip: retail comparison and visual/navigation acceptance.

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
effects/animation, gifts/attachments, discard/reorder and sent-mail browsing
remain follow-ups. No unavailable action is presented as working. M4 remains
open; final retail visual/interaction acceptance belongs to Philip.
