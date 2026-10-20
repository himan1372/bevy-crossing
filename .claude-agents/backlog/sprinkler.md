# Sprinkler (nog_sprinkler)

Source: `src/furniture/ac_nog_sprinkler.c` (`fNS_ct/mv/dw/dt`), audited in
[docs/furniture-callback-audit.md](../../docs/furniture-callback-audit.md) §5.
Profile index 0x4DA, item 0x3368, interaction 0x1000 (START_DISABLED). The
saved switch seeds a local on flag; water alpha fades to 100 or 0 in steps of
4 and a new switch change is handled only once alpha settles. Transitions emit
0x16/0x17; on state supplies level sound 0x5B; the destructor copies local
state back to the switch bit (save/lifecycle boundary). Planned, not started.
