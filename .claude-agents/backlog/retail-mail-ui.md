# Retail mail UI

2026-10-16, codex. Planned only; no implementation or Philip acceptance.

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
