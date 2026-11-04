# halloween-normal-talk — Halloween slice 3: the ordinary talk

`aHWN_norm_talk_request` / `aHWN_set_norm_talk_info`: after the forced call, a talk with a costumed villager is
the single line `MSG_2479 + looks` (no choices). Before the greeting, a talk the player starts is the forced
greeting (the approach think's talk request is `aHWN_force_talk_request`).
- [ ] halloween.rs `intercept_talk` (QuestInputSet, after `interact`): takes the `TalkEvent` of a costumed villager,
      claims it (`QuestClaim`) so the ordinary dialogue stays shut; releases the claim when the talk ends.
- [ ] check / clippy / fmt; Temp run: greeted villager -> 2479 + looks; ungreeted -> 0x98A.

## Result
`halloween.rs::intercept_talk` (QuestInputSet, after `interact`): a `TalkEvent` on a costumed villager is ours.
Greeted: `Step::Said(After::Close)` with `native::msg::normal_talk(looks)` (2479 + looks); not greeted: the forced greeting.
It sets `QuestClaim` so `dialogue::on_talk` skips the event; `end` releases it and logs "Halloween: the talk ends".
Temp run with the disc: after Tad's first talk (candy), facing him and pressing E gave message 0x09B2 (= 2479 + 3),
"Hey, there! Have you received lots of candy, sluuuurp?" then a second page "No one's gonna give you any candy if
you're not in-costume, sluuuurp!"; no ordinary dialogue box overlapped.
Not driven: the final close of this line (same `After::Close` path as the other talks), the ungreeted-by-E path.
QA traps: a villager on a cliff ledge cannot reach the player (the chase is plain run-at); pressing E facing a door
after the talk enters the house (exit: KeyS then E on the mat); `cargo check` misses Bevy query conflicts (B0001).
check / clippy -D warnings / fmt pass; no tests; real save untouched.
