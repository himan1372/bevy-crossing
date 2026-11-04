# gracie-car — Gracie's car (`s_car`)

Source: `ac_s_car.c`, `ac_s_car_draw.c_inc` (`aSCR_set_prim_col`), `ac_s_car_move.c_inc`, `ac_ev_designer_schedule.c_inc`
(`aEDSN_think_init_proc`: she stands at the car + (1, 2) cells).
The car is `s_car_DL_model`, drawn with prim colour per channel `remaining * (150, 120, 100) + done * 255` (done = presses / 100),
so it goes from dull to white while it is washed, and keeps the wash while it stands.
- [ ] gracie.rs: the car spawned two cells north and one west of her with the disc display list, its colour from the wash.
- [ ] check / clippy / fmt; Temp run: the car in a screenshot, then after a wash (white), then a partial wash.
Not in this slice: ground offsets (the car's collision / bumps), the shadow, the sparkles at 100 presses, the player's
wash animation.

## Result
`app/src/gracie.rs`: `GracieCar` (variants of the display list's materials at 0, 20 .. 100% of the wash, the part entities,
the shown level), `car_colour` (`aSCR_set_prim_col`), spawned with Gracie from `s_car_DL_model` via
`disc_town::build_model_full`, two cells north and one west of her, and removed with her; `car_colour_update` swaps the
materials by `Talk.car_presses`, which `wash` keeps (the colour stays after the game, as `button_presses` does while the car
stands). Gracie's own spot is now searched so that her cell and the car's 3 x 3 cells are clear ground (the first try put the
car half over the river).
Temp run with the disc (synthetic save, calendar DESIGNER, clock to noon): "Gracie's car stands at (17.5, 3.0, -43.5)", a red
convertible against the plaza fence, dull at the start, noticeably brighter at 51 presses (screenshots); the wash counter
over it.
Not driven: the 100% colour (the formula gives 255 on every channel), the car at the end of a winning wash.
Not built: the car's ground offsets (`aSCR_set_bgOffset`) and collision, the shadow (`obj_car_shadow`), the sparkles at 100
presses (`eEC_EFFECT_CAR_BLIGHT`), the player's wash animation; the car's facing was left as the model comes.
check / clippy -D warnings / fmt pass; no tests; real save untouched.
