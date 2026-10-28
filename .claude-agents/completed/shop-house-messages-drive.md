# Drive the statue, basement, built and offer house messages

Philip: "statue and basement messages" (2026-10-27). Drive every opening house talk on tier 0 (message and walk): REHOUSE_STATUE 0x08,
STATUE_BUILT 0x09, BASEMENT_BUILT 0xCC, UPPER_BUILT 0xD5, LARGE_BUILT 0x07, REHOUSE_BASEMENT 0xD0 (walk); REHOUSE_LARGE 0xD1, REHOUSE_UPPER 0xD4,
BASEMENT_PAID 0xCD, REHOUSE_2_OFFER 0x04 (no walk). Evidence only; fix what it shows.

## Result (2026-10-27, claude)
Driven on tier 0 (Temp, capture build, disc, synthetic house records, game clock 15:00, a fresh process per case). Opening talk code, message, Nook z
(start -2998.47, walk limit -2997.5):
| House record | Talk | Message | Walk |
|---|---|---|---|
| size 3 (upper), loan 0, no statue ordered | REHOUSE_STATUE 0x08 | 0x1083 | walked |
| statue ordered, next size statue | STATUE_BUILT 0x09 | 0x1084 | walked |
| renewal, basement ordered (size 2) | BASEMENT_BUILT 0xCC | 0x2EB2 | walked |
| renewal, size 3 | UPPER_BUILT 0xD5 | 0x2EBB | walked |
| renewal, size 2 | LARGE_BUILT 0x07 | 0x1082 | walked |
| size 2, no basement, loan 0 | REHOUSE_BASEMENT 0xD0 | 0x2EB6 | walked |
| size 2, basement | REHOUSE_LARGE 0xD1 | 0x2EB7 | stayed |
| size 2, basement billed | REHOUSE_UPPER 0xD4 | 0x2EBA | stayed |
| size 1, basement | BASEMENT_PAID 0xCD | 0x2EB3 | stayed |
| size 1, no basement | REHOUSE_2_OFFER 0x04 | 0x107F | stayed |
Every case matches `aNSC_set_talk_info_start_wait(1)` and the SAY_HELLO_APPROACH choice in `aNSC_start_wait`. No WARN/ERROR; no code changes.
Not driven: the follow-up conversation steps after each opening line (expansion, roof colour, basement orders), which earlier tasks covered.
