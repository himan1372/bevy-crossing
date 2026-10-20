# Player home door interaction

Status: implemented, Philip acceptance pending.

Philip reports huge interaction area prevents facing mailbox/gyroid without entering house.

- [x] Exterior player door now uses 1.25-unit facing-ray reach to the finite panel, from outside its wall. Replaces 3.2-unit nearest-point/wide-cone gate for this entry only.
- [x] Physical collision/front approach, mailbox and interior/villager door gates untouched. No-disc player door also uses the tighter aimed panel.
- [x] Workspace check/clippy/fmt/capture/default build pass; no automated tests added/run.
- [x] Disposable runtime: left-hand mailbox opened from street side (player about -1.73,-28.35), and from its requested left edge (-3.18,-27.24) after walking toward it. Screenshot interaction-left-edge.png shows mail UI; log Mailbox opened, no Entered house. Direct nearby diagonal door approach still entered house (player moved to interior 3000,1,3001.4). Images/logs stay in Temp/bevy-crossing-mailbox-review. Preview closed, save SHA256 unchanged.
- [ ] Philip acceptance.

Gyroids currently have rendering/animation but no conversation handler. This change prevents door E capture near them; it does not implement gyroid dialogue. Recorded explicitly instead of claiming talking verified.

Next requested task after releasing this claim: missing wishing fountain; source/layout/disc model located as WISHING_WELL / ac_shrine, actor shifts +20 x, -19 z game units.
