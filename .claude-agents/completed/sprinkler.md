# Sprinkler (nog_sprinkler)

Source: `src/furniture/ac_nog_sprinkler.c` (`fNS_ct/mv/dw/dt`), audited in
[docs/furniture-callback-audit.md](../../docs/furniture-callback-audit.md) §5.
Profile index 0x4DA, item 0x3368, interaction 0x1000 (START_DISABLED). The
saved switch seeds a local on flag; water alpha fades to 100 or 0 in steps of
4 and a new switch change is handled only once alpha settles. Transitions emit
0x16/0x17; on state supplies level sound 0x5B; the destructor copies local
state back to the switch bit (save/lifecycle boundary). Planned, not started.

## Implementation (claude, 2026-10-19)

logic::furniture_sprinkler::Sprinkler (fNS_ct/mv: counter wrap 1024, fade
first, toggle only when settled, toggle sounds; water alpha byte; tile-1
scroll value). app::furniture::sprinkler: body + water anchor from disc lists,
E through the generic silent branch (switch_changed = bit differs from the
previous tick), 60 Hz ticks in the player room, se_at 0x16/0x17 with the
birth/removal gate, sound_on feeding level 0x5B into ambience, water
materials' env (0,180,255,alpha), fNS_dt write-back via PlayerRoom::set_switch
when the room is left. Placement already starts with switch 0 (START_DISABLED
semantics). No save schema change.

## Verification (Temp only; no tests)

Real save C15154C0... unchanged. Preview, sprinkler at cell 50: off at
placement; E,E (second 9 frames in) leaves switch 0 with water on (local on);
third E sets switch bit 17 with water off; fourth E switch 0, water on; after
leaving the house F5 saves bit 17 set (write-back); re-entry shows water on.
Screenshots inspected; ambience peak nonzero while on, nonfinite 0, no
WARN/ERROR. Gates: check, all-target clippy -D warnings, fmt, capture, default
41s, release 3m02s.
Limits: water flow (TEXEL1 scroll) not rendered; F5 inside the room while
local and switch disagree saves the switch bit (retail saves on scene exit).
