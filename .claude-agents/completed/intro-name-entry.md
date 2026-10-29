# New-game intro, slice 2b: the name windows and Rover's questions

Part of the intro demo. Source: `m_ledit_ovl.c` (`mLE_win_data`, `mLE_set_dl`, `mLE_ledit_ovl_init`), `m_editor_ovl.c`
(`mED_all_space_check`, `mED_ng_word_check`, single-line editing), `ac_npc_guide_move.c_inc` (`aNGD_*`), messages 0x2AC6-0x2AE6.

Script rules read from the source and the disc text (control codes sized from `mFont_cont_info_tbl`):
- Order NPC0 index 9 value 1 at the end of a message means "open the next window"; which one follows the stage: 0x2AC6 -> time menu,
  0x2AC7 -> player name, 0x2AD8 -> town name. After a window: time 0x2AE5, player 0x2ACA, town 0x2AE4.
- 0x2ACA's answers ("Isn't it cool?" / "Isn't it cute?" / "No! No! No!") lead to 0x2AD6 / 0x2AD7 / 0x2AE3; 0x2AD6 / 0x2AD7 ask
  ("You know it!" / "I'm not a boy!"; "I forgive you." / "I'm not a girl!"). Gender: male when (cool) == (the first answer), which is the
  four-way table of `aNGD_sex_select_wait2` (order 1 from 0x2AD6, 2 from 0x2AD7). 0x2ADF / 0x2AE0 are followed by 0x2AD8 (`aNGD_sex_select_after`).
- `aNGD_check_talk_msg_no`: showing 0x2AC9 / 0x2ACD / 0x2ACF / 0x2AD3 sets face-choice bits 3 / 2 / 1 / 0.

## Done (2026-10-29)
- `app/src/name_entry.rs`: the name window (`nam_win_*` for the player, `mra_win_*` for the town) from the disc, title and field text at the
  source's offsets, the window slides in from the top and the shop editor's keyboard from the bottom (`Slide`), single-line `Field<8>`,
  end refused for an empty name and for the disc's banned words (`0x74A..` strings, error sound 0x1003, field wiped). `InputFocus::NameEntry`.
- `app/src/intro_train.rs`: Rover's script follows the choice codes; an order (9, 1) at a message's end opens the window for the stage; the name
  goes into `PlayerIdentity` (player / town) and the next message is the one the source names; gender and face bits are tracked.
- Driven (Temp capture copy, `BEVY_CROSSING_INTRO=play`): 0x2AC6 -> 0x2AC7 -> 0x2AC8 -> the window opens, "hi" typed on the keyboard, F4, the log shows
  "named: hi / woponese" and Rover goes on with 0x2ACA ("Isn't it cool? / cute? / No!"). Screenshots seen: the window with the keyboard, the typed name, 0x2ACA's question.

## Not done
The time menu (TIMEIN; "No" on the time check, 0x2AE1 / 0x2AE6), the town-name window was not driven (same code with the `Country` kind), using the gender and
face (`PlayerLook` has no female faces yet), the rest of the script after the town name (the moving questions, standing up, the phone call, the last talk, the scene change),
Rover's expressions and animations, the pencil's colour / window scroll polish, the window's own sound and the keyboard hints text.
