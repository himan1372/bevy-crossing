# Museum room doors fade

Philip: "doors" (2026-10-28), from the handoff gaps: the museum's inner room doors still cut; the door
sound plays before the fade; a villager's house door fade not driven.

- Museum lobby <-> exhibit room doors are not state changes, so `scene_wipe::doors` never saw them:
  `SceneWipe::hold` / `release` let `museum::doors` fade out, apply the room change at black, fade in.
- Door sound: the cue list (`door_cues`) starts on the frame the door fires, the same frame the fade
  starts (`doors` in `Last`), so the two are already in step; nothing changed. The cue ticks (2..98)
  span the fade out, hold and in.
- Drive the villager house door and the museum doors in a Temp capture copy.

## Done (2026-10-28)
- `SceneWipe::hold` / `release` (input frozen while held); `museum::doors` stores the room change and applies it at black. Driven (Temp capture copy, hour 14, door position read from the log): lobby -> gallery logged `scene wipe: out`, `museum room 0 -> 1`, `scene wipe: in` 0.51 s apart. Return doors share the code, not driven separately.
- Door sound: no change (see above): cue ticks start on the same frame as the fade.
- Villager house door: still not driven (needs a villager's door position; `house_enter` shares the `HouseState` path the player's house door used).
