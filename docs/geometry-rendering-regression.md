# Animated geometry investigation — 2026-11-15

The user reports stretched triangles or broken character shapes during both
cutscenes and gameplay. The saved
user run uses the November 8 release, DX12/Visual, windowed 640x480, and
reaches the opening Daventry cutscene. Its normal exit establishes process
success only. The final M4 fullscreen test exercised menus, not gameplay
geometry. The earlier missing exclusive 640x480 mode in Remote Desktop is
a separate issue.

Implementation session B uses our source, the exact locked dependencies and
original data files; no executable study or visual inspection is involved.

## Baseline evidence

Two bounded native runs use the unchanged release, original Daventry data,
`--windowed --sound off --no-ai --check-pack`, and the same automated menu
sequence at world update 120. Each captures its ordinary save thumbnail,
saves in an isolated directory and quits normally. No original files or
player saves are written. Numerical comparison only; images were not viewed.

| Check | DX12 | Vulkan |
| --- | --- | --- |
| Normal exit | 14.78 seconds | 12.98 seconds |
| CPU pack check | 21,440,085 corners | 21,440,085 corners |
| Largest position difference | 0.0080 game units | 0.0080 game units |
| Largest normal difference | 0.000026 | 0.000026 |
| Texture-coordinate / part-ball difference | 0 / 0 | 0 / 0 |
| Shader queue empty before thumbnail | Yes | Yes |

Saved world, position, heading, actor tilt, camera, journey and panels match
exactly. The 160x120 thumbnails differ at 12,958 of 19,200 pixels, with a
mean RGB-channel absolute difference of 8.7066/255 and maximum 71/255.
This establishes renderer divergence at the same saved state, not which
image is correct or what causes the divergence.

Local evidence is under the ignored `kq8-artifacts/geometry-investigation/`:
`viewer-20261115-063035.log`, `baseline-dx12/`, `baseline-vulkan/`, and
`scene_compare.py`. The latter launches each child with a 55-second limit.

## Rejected hypotheses

The packed animation shader carries point, normal and next-frame indices
as integer bit patterns in float lanes. Source inspection found consistent
layouts; the numerical vertex-stage probe below preserved these bits.

A second, stronger candidate is the locked WGPU 30.0.1 DX12 draw-count path:
ordinary indexed indirect drawing supplies special constants for the first
instance, while indexed indirect drawing with a GPU-supplied count uses the
shared command signature after setting those constants to zero. Our
character shader uses the instance index for both animation tags and world
transforms. A wrong index could therefore break parts even when their packed
source data is valid. This candidate was reproduced in isolation but rejected
as the scene's cause: Bevy already excludes this path on DX12.

### Reproduced dependency bug already avoided by Bevy

The offscreen probe reproduced the indexed indirect-count defect on the
same AMD GPU with optimized DX12/FXC. Direct and ordinary indirect calls
returned 0, 17 and 131071 as requested; the count variant returned 0 for all
three, including the wrong storage-buffer tag lookup. Vulkan returned the
expected index and tag through all three paths. The run took under one
second across the two backends; no window or image inspection was used.
Evidence: `probe-baseline.stdout.log` and `probe-baseline.stderr.log`.

The production node-decoding function preserved the point/normal/next index
bits and all tested positions on both backends. The synthetic normal value
-32768 decoded to -1.0000305 on DX12 rather than the expected -1; Vulkan
clamped it correctly. This is a separate small endpoint discrepancy, not
evidence of corrupted position indices. Production `pack_normal` uses
the -32767..32767 range. The raw probe therefore reports an overall failure
for both the large instance-index defect and this synthetic endpoint; neither
failure has been hidden by changing the expectations. The initial probe
build also retained two WGPU API compile errors, corrected before execution.

An initial feature-exclusion trial passed four focused checks in 0.55s
(release build 2m11s, test build 1m33s), but the 18.38-second scene run
produced a thumbnail byte-identical to the old DX12 output. Inspection then
found locked Bevy's `render_phase/mod.rs` already excludes the count path
on DX12, referencing [WGPU issue 7974](https://github.com/gfx-rs/wgpu/issues/7974).
The redundant renderer change and policy tests were removed. Trial evidence
is retained in `corrected-auto/` and `corrected-comparison.json`; these names
record the attempted correction, not a successful geometry fix.

## Confirmed character-shader defect

`object_light.wesl` passed its vertex-input struct into `model_corner`,
which read `v.instance_index` inside the helper to choose an animation tag.
The locked DX12 compiler applies a draw's first-instance correction at
entry-point builtin reads, but loses it inside the struct helper. The outer
vertex function still reads the correct world transform, so a part can
combine its own placement with another part's pose.

Bevy's own `forward_io.wesl` passes the index separately to its helper for
this reason. Its referenced upstream report is now
[WGPU issue 4564](https://github.com/gfx-rs/wgpu/issues/4564).

The numerical direct-draw probe reproduced that exact pattern: for first
instances 17 and 131071, the entry point read the correct index, the struct
helper selected instance zero's tag, and the scalar-argument helper selected
the correct tag. Vulkan selected the correct tag in all cases. Evidence:
`probe-struct.stdout.log`. This probe does not use the rejected count path.

The production fix passes `v.instance_index` explicitly from the vertex
entry point to `model_corner`, then uses that scalar for animation-tag
lookup. Both static and animated specializations retain a matching helper
signature. Renderer selection/presentation, packed data, models, textures
and animation timing are unchanged. The old shader-queue message is also
corrected: an empty queue does not prove correct geometry.

## Verification

The shader correction's release build passes in 1m42s; the test binary
builds in 1m28s. All three focused renderer tests pass in 0.29s, including
nine exact hardware position readbacks using the actual production helper
and call site. Direct, ordinary indirect and Bevy's three-command fallback
each select the correct poses at first instances 0, 17 and 131071. GPU
polling/readback has a 15-second deadline and the local test parent a
60-second limit. No game assets are embedded in the regression.

The corrected production-default DX12 scene run passes in 14.73s with
identical saved world/camera/position state and shaders ready before the
capture. Against the existing Vulkan baseline, only 22/19,200 thumbnail
pixels differ (99.885% exactly match); mean RGB-channel absolute difference
falls from 8.7066 to 0.00349/255, with maximum 8/255. The corrected output
differs from old DX12 at 12,958 pixels. Evidence: `shader-fixed-auto/`,
`shader-fixed-comparison.json` and `shader-renderer-tests.stdout.log`.
This is strong numerical evidence of the corrected production rendering,
not a claim of full-resolution pixel identity or complete original-game
fidelity. Full campaign, the exact opening cutscene as a separate capture,
and visual inspection are **not tested** by this focused correction.

From the repository, the reproducible automated check is:

```powershell
cargo test --release --locked -p kq8-viewer --bin kq8-viewer renderer:: -- --include-ignored --nocapture --test-threads=1
```

It needs Windows and a hardware DX12 adapter advertising the features under
test. It requires no game assets or window. Ordinary test runs skip the
hardware-specific case; the existing renderer policy checks still run.

The shared executable is rebuilt at
`C:\Users\Robert\Documents\Codex\KQ8\KQ8-REA\target\release\kq8-viewer.exe`.
For Command Prompt in the current Remote Desktop session:

```bat
"C:\Users\Robert\Documents\Codex\KQ8\KQ8-REA\target\release\kq8-viewer.exe" --game "C:\Users\Robert\Documents\Codex\KQ8\King's Quest 8 GOG" --windowed
```

Expected behavior: animated parts use their own pose indices in cutscenes
and gameplay. Windowed launch avoids the separately
reported missing exclusive 640x480 mode. This command is optional launch
guidance, not a new manual milestone gate.

No further successful unrelated checks were repeated. The first rejected
feature-exclusion trial is retained above alongside the successful fix.
