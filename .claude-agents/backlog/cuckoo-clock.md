# Cuckoo clock (sum_log_hatoclk)

Source: `src/furniture/ac_sum_log_hatoclk.c` (`aSumLogHatoclk_*`), audited in
[docs/furniture-callback-audit.md](../../docs/furniture-callback-audit.md) §6.
Profile index 0xC1, item 0x1304. A three-state mechanism starts on minute 0
(hour-count table, noon and midnight = 12) or minute 30 (one call),
alternates stop/play and rearms only after leaving both minute windows;
entering during a trigger minute can start it. Sound 0x133 at frame
GETREG(MYKREG,33)+2 (resolve the register default). Reuse the native clock
kernels (logic::furniture_clock) and the rigs/clocks skinned builders.
Planned, not started.
