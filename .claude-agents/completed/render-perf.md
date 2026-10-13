# render-perf — 60 fps in a release build (recorded after the fact)

Done by claude 2026-10-13 without a claim; recorded so the board matches.

- [x] Measured with vsync off (BEVY_CROSSING_NO_VSYNC=1): release ~300 fps, dev ~260 fps.
- [x] Dependencies at opt-level 3 in dev (dev was 57 fps).
- [x] Gate (b1a1e44).
