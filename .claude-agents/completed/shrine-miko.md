# Shrine events, slice 1: Miko (New Year's omikuji)

Scope: `SP_NPC_EV_MIKO` at the shrine block unit (3,3) while NEW_YEARS_DAY is active (Jan 1, 06:00-10:00); a talk in
front of her: 0x190C offer (50 Bells); full letter slot 0x190F, too poor 0x190E; else RANDOM(4) result 0x1911+n, the
destiny of the table, a fortune-slip letter into the mail (header 0x72+RANDOM(3), four random strings, the result
string). Not in this slice: Tortimer's own shrine speeches (soncho2: commemorative items, trophies), the five
visiting villagers, the groundhog cutscene (separate task).

- [ ] app/src/miko.rs, tortimer.rs shrine helper, main.rs
- [ ] gates, Temp run Jan 1, result, comms, commit

## Result
`app/src/miko.rs` (+ `tortimer::shrine_cell`). Run in game (Temp save, disc, Jan 1 07:50): the shrine maiden (her own
disc model) stands at the shrine block's unit (3,3); a talk from the side gets 0x1916; from in front 0x190C (Yes/No),
Yes: result 0x1914/0x1913, destiny set (5 = goods luck), 50 Bells taken, the fortune slip handed over and a letter
("You're in for a great year! Luck / Love / Wealth / Health" lines from the string banks) filed in the player's letters.
Unverified: the full-letters (0x190F) and too-poor (0x190E) refusals, the other results, No.
Not built: the New Year table, the five visiting villagers, Tortimer's shrine speeches (soncho2: commemorative items,
trophies), her turn toward the point after a talk, the destiny's effects. The groundhog (majin) is the next slice.
