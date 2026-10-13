# Fix letter notification that never fades
- [x] The mail notification text stays visible indefinitely. NOTIFY_DURATION (3.0s) logic looks correct in app/src/mail.rs — do not guess; inspect live.
- [x] Diagnose via BRP (port 15702, build with the `remote` feature): watch MailNotifyTimer and MailNotifyText visibility over several seconds.
- [x] Fix the timer/visibility handling so the notification fades after 3 seconds.
- [x] Ask Philip to playtest on Windows and confirm.
