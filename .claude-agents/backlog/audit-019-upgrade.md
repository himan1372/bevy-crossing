# Audit the 0.19 upgrade against the bevy-019 skill
- [ ] Read app/Cargo.toml and logic/Cargo.toml — confirm bevy version resolution, edition, MSRV
- [ ] Scan app/src and logic/src for pre-0.19 API patterns (EventWriter/EventReader, bare Handle<Mesh>, get_single, f32 font_size, RenderGraph/Node boilerplate)
- [ ] Verify flagged APIs against docs.rs/bevy/0.19.1
- [ ] Report findings on comms.txt; fix what's safe via repo edits, flag what needs a local build/test
