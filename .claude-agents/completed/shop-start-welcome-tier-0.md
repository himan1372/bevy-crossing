# Drive the tier-0 sale-day welcome

Philip: "do 1" (2026-10-28): drive the sale-day welcome (EVENT 37) on tier 0 with the balloon already claimed and no house talk, the one start case
`completed/shop-start-audit.md` and `completed/shop-start-tiers-1-3.md` did not cover. Evidence only.

## Result (2026-10-28, claude)
Driven in Temp (capture build, disc, synthetic tier-0 save with loan 1, balloon already claimed, game clock 15:00, fresh process): the first shop message
was 0x10A0 (EVENT 37 + 0x107B), with no balloon or house talk before it; Nook sat at the source walk limit z -2997.5 (260 units) in every sample
(the first sample came after the approach walk; the start z -2998.47 was shown by earlier tier-0 runs). No WARN/ERROR. No code change.
This closes the last unchecked sale-welcome case: tiers 0-3 now match `aNSC_get_start_call_msg_no`.
