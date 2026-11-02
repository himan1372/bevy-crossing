# Retail HUD follow-up

Philip selected the source-faithful HUD audit from the handoff. Existing disc artwork and kernels already implement clock/date and pocket Bells; development labels belong behind F10. Original `m_banti.c` excludes first intro, suppresses display during demos and submenus, and selects sides using the player's native screen position.

Use the existing renderer and kernel. Replacing the HUD would discard working native art; adding persistent Bells/context labels has no support in the audited original draw path. Correct the adapters instead: suppress intro drawing, pass all modal focus states, advance presentation with real time during PC pause, and convert widescreen projection into centered 320x240 coordinates. Keep world simulation paused. Correct the outdated dev-overlay module description.

Preserve numeric inventory/save data, runtime disc loading, usable missing-disc fallback, and existing F10 behavior. No new assets/dependencies/tests. Validate native clock idle/movement, modal open/close, intro, time/date changes, side switching, pocket Bells and no-disc operation in disposable Temp scenarios. Acceptance and M4 remain open.

## Philip's added inventory scope

Restore the missing fish/insect collection pages using original frames, tabs, second icon palettes, names and native 8x5 order. Record actual successful catches in existing saved furniture collection bits, independently of museum donations or generic pocket possession. Earlier unrecorded catch history cannot be reconstructed reliably.

Make the wallet target interactive: normal Bell bags can be deposited in full or refused at the 99,999 cap; affordable native denominations100/1000/10000/30000 can be withdrawn into a free pocket. Full pockets refuse without mutation. Preserve wrapped/quest conditions. Use the native42-tick signed-step display counter and input wait, while actual wallet/items commit atomically. Opening/reloading must use committed state and clear transient presentation. Custom design/editor support was asked about and confirmed absent; no implementation was requested.
