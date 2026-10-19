# Isolated BRP launch and request shapes

Read for preview launches or raw HTTP requests. These examples match the checked Bevy 0.19.1 source and installed extras 0.22.10. The manifest's 0.22.1 requirement is a compatible range, not an exact pin. Recheck resolved source or `rpc.discover` after dependency changes or schema errors.

## Launch

Build from the repository with `cargo build -p bevy-crossing --features capture -j 1` when needed. Create a unique Temp directory, place only synthetic scenario data there, then start the absolute executable with that directory as `WorkingDirectory`. Example PowerShell template; substitute verified paths:

```powershell
$previewDir = Join-Path $env:TEMP ('bevy-qa-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $previewDir | Out-Null
$previewExe = 'C:/absolute/checkout/target/debug/bevy-crossing.exe'
$discImage = 'C:/absolute/user-disc-image.iso'
$preview = Start-Process -FilePath $previewExe -WorkingDirectory $previewDir -ArgumentList @('--disc', ('"' + $discImage + '"')) -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $previewDir 'stdout.log') -RedirectStandardError (Join-Path $previewDir 'stderr.log')
```

`-WindowStyle Hidden` applies to launch; the game still needs a usable rendered surface for primary-window captures. Inspect black captures before diagnosing game art. Keep the returned PID and verify executable identity before closing it. Never stop all game/Rust processes. For a no-disc scenario, omit the flag and ensure the disposable directory has neither configured settings nor a discoverable `rom/` image.

BRP normally listens on `http://127.0.0.1:15702`. Check for an existing listener first and identify its owner; do not accidentally attach to Philip's game or start competing previews. Wait for readiness and inspect startup errors rather than rebuilding repeatedly.

## Requests

POST JSON-RPC with `Content-Type: application/json`. Example envelopes:

```json
{"jsonrpc":"2.0","id":1,"method":"rpc.discover","params":{}}
{"jsonrpc":"2.0","id":2,"method":"world.query","params":{"data":{"components":["bevy_ecs::name::Name","bevy_transform::components::transform::Transform"]}}}
{"jsonrpc":"2.0","id":3,"method":"brp_extras/send_keys","params":{"keys":["KeyW"],"duration_ms":100}}
{"jsonrpc":"2.0","id":4,"method":"brp_extras/screenshot","params":{"path":"C:/absolute/Temp/bevy-qa/capture.png"}}
```

Each line is a separate request. Inspect JSON-RPC `error` even after HTTP 200. A query returns entity IDs and component maps. Re-query/disambiguate after scene/reload, and keep IDs as exact integers (avoid floating-point loss in JavaScript). Use only reflected components.

For an entity ID obtained from the query, the parameter shapes are:

```json
{"entity":123,"components":["bevy_transform::components::transform::Transform"]}
{"entity":123,"components":{"bevy_transform::components::transform::Transform":{"translation":[0,0,0],"rotation":[0,0,0,1],"scale":[1,1,1]}}}
```

Use the first with `world.get_components`, the second with `world.insert_components`; `123` is a placeholder, not a reusable entity. Returned component values live under `components`. A transform edit does not necessarily update facing or gameplay-owned state. Avoid replacing unrelated components or mutating the real game.

Known request shapes do not need rediscovery before every call. For an unknown method/type or schema error, inspect discovery and the matching local implementation once; do not retry guessed historical `bevy/query` aliases.

`send_keys` schedules a simultaneous press/hold/release, not a completed gameplay action. Wait for release and observed state before another dependent action; allow full animation time and dismiss notices before QuickLoad. Screenshot requests take a destination path and publish a PNG; they do not return image bytes for the client to save. Inspect the file before claiming visual success. Keep all capture paths inside the disposable directory.
