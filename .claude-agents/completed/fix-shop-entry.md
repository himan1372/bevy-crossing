# Fix shop entry (E key silently rejected)
- [x] Read the door-interaction code in app/src/: entry is rejected when a tool is equipped, mail/quest UI is open, dialogue is active, or the player is too far away / not facing the door.
- [x] Diagnose live via BRP (port 15702, build with the `remote` feature): inspect ShopBuilding.door_pos, player Transform, EquippedTool, MailUi.open, QuestUi.open, DialogueState.
- [x] Fix the rejection cause — E should enter the shop when the player faces the door with empty hands.
- [x] Ask Philip to playtest on Windows and confirm.
