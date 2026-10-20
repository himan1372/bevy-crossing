# Normal/release compiler memory failure

2026-10-17, codex. Philip reports cargo run --release failing in rustc allocation
and trait normalization. Capture build succeeds; default development build also
failed during the preceding task. Investigate memory/commit availability and
feature-specific compiler behavior before changing build policy or machine settings.

- [x] Claim; inspect source/config/toolchain and current memory evidence.
- [x] Apply a justified bounded fix or identify the concrete external blocker.
- [x] Appropriate default/release builds, sequential Cargo. No source changed;
  prior check/lint/format evidence is unchanged. No tests added/run without
  Philip request. Do not run the actual save.
- [x] Record evidence and limits; no codex-handoff.md edits at Philip's request.

Preserve actual save, settings and unrelated .gitignore. M4/migration remain open.

## Current diagnosis

Rust1.99.0 stable. Default/capture source differences are confined to additive
RemotePluginGate; no changed build settings. Windows GlobalMemoryStatusEx shows
15.88GiB RAM with2.34GiB available, commit limit63.88GiB with3.02GiB available.
Get-Process PrivateMemorySize64 identifies SteelSeriesCaptureSvc PID26448 at
20.25GiB and PID13580 at19.68GiB, while each has about1.1GiB working set. These
are unrelated user capture processes. During a sequential release build rustc
reached2.22GiB private allocation. This is evidence of machine commit pressure,
not proof of a compiler regression or a source bug. Asked Philip before stopping
the capture processes because recording could be interrupted. Philip explicitly
approved "Stop both and retry". Verified both process identities and stopped only
13580/26448. Immediately afterward RAM available4.49GiB; commit limit39.89GiB and
available18.88GiB (Windows changed the dynamic commit limit). No service/startup,
page-file or compiler configuration was changed. A pre-stop release build failed;
the same cargo build --release -j1 then passed in4m03s with no code/settings
change. SteelSeries restarted capture processes23256/29224; their observed
private allocation is about0.44GiB each. Those new processes were left running.
This confirms the failing build could complete after releasing the excessive
capture allocations. The normal cargo build --workspace -j1 also passed in1m27s.
Both current executables include the F4 cabinet access. No actual game process
was launched for this investigation, so the real save was never loaded.

No source, manifest or compiler-policy edit is justified by the current evidence.
No actual-save launch, tests or codex-handoff.md edits. Actual save current-turn
SHA256 remains DD89745CCFEDDEAEA619A1C931FEE5233EC533C1860E3B5A910A00B8121E5721;
user settings and unrelated .gitignore preserved.

This resolves the current build blocker; it does not fix or establish the cause
of SteelSeries' excessive allocation. No unrelated user processes were stopped.
