# Fix letter notification that never fades
- [ ] The mail notification text stays visible indefinitely. NOTIFY_DURATION (3.0s) logic looks correct in app/src/mail.rs — do not guess; inspect live.
- [ ] Diagnose via BRP (port 15702, build with the `remote` feature): watch MailNotifyTimer and MailNotifyText visibility over several seconds.
- [ ] Fix the timer/visibility handling so the notification fades after 3 seconds.
- [ ] Ask Philip to playtest on Windows and confirm.
