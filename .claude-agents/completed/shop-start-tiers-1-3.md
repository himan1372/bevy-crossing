# Drive tiers 1 to 3 for the non-twin start behaviour

Philip: "drive tiers 1 to 3" (2026-10-27): ordinary greeting, forward walk (limits 260 / 420 / 500) and the sale-day balloon talk (218, depart 220)
on the convenience store, super and department store (lower floor), from `completed/shop-start-audit.md`.

## Result (2026-10-27, claude)
Driven in Temp (capture build, disc, synthetic tier 1/2/3 saves, game clock 15:00, a fresh process per case), lower floor of each:
- Ordinary greeting (loan 1): message 0x1091 on all three tiers; Nook walked to the source limit: final z -2997.5 (tier 1, 260 units),
  -2995.5 (tier 2, 420) and -2995.5 (tier 3, 500), which equal SHOP_ORIGIN.z + limit/40 - centre.z for each layout.
- Sale-day balloon (event active, balloon unclaimed, free pocket, house talk pending): first message 0x2EC0 (218) on tiers 1 and 2, 0x2EC2
  (220, `BALLOON_DEPART`) on tier 3; no house talk or welcome before it; no walk (z stayed at the start values -2998.44, -2996.5, -2996.44).
- Sale-day welcome (balloon already claimed, loan 1): 0x10A0 (37) on tiers 1 and 2, 0x108B (16, `EVENT_DEPART`) on tier 3; all walked to their limits.
- House talk walk: built-house talk (0x03, 0x107E) walked to the limit on tiers 1, 2 and 3; the small-house offer (0x00, 0x107B) on tier 3 stayed put.
No WARN/ERROR in the logs. No code changes in this task.
