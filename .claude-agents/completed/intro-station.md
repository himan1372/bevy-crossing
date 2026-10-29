# New-game intro, slice 3a: the station building

Philip: "ok" to starting slice 3, the arrival in town (2026-10-29). The town already has the rails, the brick platform and the map sign at the station acre (2, 0) but no station building.

## Done
- `ac_station.c`: the station is `cKF_bs_r_obj_{s,w}_station{1,2,3}` (TRAIN_STATION = 0x5809 in the layout) with the palette `aSTR_PAL_STATION{1,2,3}_{A..E}` (42 / 47 / 52 plus the colour).
  Three more `BUILDINGS` specs (parts t1, t2, t3, short, long), `place_buildings` picks the model by `station_type / 5` and the colour by `% 5` (`RetailBuilding.palette_shift`).
- `disc_town::station_type(seed)`: `mSDI_StartInitNew`'s `RANDOM(15)`, derived from the town's seed so a town keeps its station without a new save field.
- Seen in a town run: the brick station with its balustrade, pillars and stairs on the station acre (origin (0.5, 4.0, -42.5)), the platform's railing.

## Next (3b ..)
The arrival: the scene change from the train, the train (`train0` / `train1`, `m_train_control` kernel in the pinned Rustimal `train.rs`), the player standing in it, the station master (Porter) and his talk, the ride-off and the walk out.
