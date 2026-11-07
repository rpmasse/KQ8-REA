# Authored Rock aim, flight and recovery

Implementation B, 2026-11-07. Source: A's
[rock-projectiles.md](behaviour/rock-projectiles.md), Q202. This replaces the
former gravity-based approximation. Original runtime behavior and native
gameplay are **not tested**; implementation checks use the installed data,
numeric cases and Rust script/collision dispatch.

The held-Rock path clips the pointer ray to drawn terrain before picking with
mask 0x29. An accepting object's own box must be strictly within the Rock
file's range of Connor's feet. That object's queued action receives the held
identity and type and owns any throw. Direct throws retain the actual ray
point, request a turn only in third person, and enter ThrowRock immediately.
The ordinary post-script turn respects suppression and permits ThrowRock.

The delivered do cue reads the selected frame's full named Rock part. Its
translation is transformed by Connor; its authored rotation stays unchanged
through flight and final script placement. The local launch height remains
separate from world launch height. Only ordinary ThrowRock supplies kind4
and current ranged damage; Tile2 uses its existing choreography and file kind.
The ordinary state remembers/restores its projectile and weapon bank through
away, throw and out. Launch does not deduct inventory.

The core uses the two documented height segments, f32 progress and separate
rates, range limited by horizontal launch-to-aim distance, and strict range
overrun into stationary-horizontal falling. Collision dispatch retains the
normals supplied by the existing sweep. A steep normal enters falling before
ordinary damage; floor termination uses the floor grid, not drawn water.
The final full placement reaches rockHit.cs. That script performs the normal
Die-time inventory removal/recovery, using the separately verified item-drop
continuation. Kind4 removal requests the authored placed ouchSound.

Compatibility boundaries remain explicit:

- The original falling vertical temporary is undefined. This recreation
  initializes its displacement and descends at 8,000 units/s. This is a
  safeguard, not verified original numeric parity.
- A missing named part retains the stored translation fallback and uses
  identity rotation. Zero range and malformed negative height enter the
  defined vertical fall; nonfinite launch/rate inputs are rejected. Zero
  normals provide no slope information and are not normalized.
- Saved authored arcs include their launch, local height, full rotation,
  range, progress, rates and falling phase. Older velocity/gravity arc slots
  are rejected by the explicit authored schema rather than inventing a
  conversion. Slots without such an old in-flight arc remain readable.
- Sweeps currently expose their nearest surface per collider. The projectile
  preserves supplied normals, including identical-position tied contacts,
  but does not claim new simultaneous-surface geometry precision. A
  StartedInside hit has no supplied normal and is not assigned a fake one.
- Original exceptional midflight saves/world departure remain outside the
  study's evidence. Rust round trips prove the recreation's own continuation.

## Automated check

From the repository in PowerShell:

```powershell
$env:KQ8_GAME_DIR="C:\Users\Robert\Documents\Codex\KQ8\King's Quest 8 GOG"
cargo test --locked --release -p kq8-core authored_rock_numeric_dispatch_and_asset_matrix -- --ignored --nocapture
```

Expected: one grouped test passes, logging the actual delivered cue frame's
part translation/rotation, the f32 curve samples, and ray/reach, state request,
contact/immunity, final floor placement and Die-time inventory checks. It
does not rerun the separate item registration matrix or launch the original.

The first shared core build passed. Its existing Rock puzzle/recovery and
projectile-save round-trip tests passed. The grouped matrix first stopped on
an overly exact synthetic float assertion (150.00002 versus 150); the corrected
case explicitly places progress on the boundary and tolerates f32 rounding.
Its second run passed that section and stopped on a fixture using the short
name `Rock` with `Items::count`, which requires `INVITEM_Rock`. The three count
assertions were corrected. These failures did not change the authored flight
math. The final grouped test **passed** in 0.13s after the coordinated build;
log: `kq8-artifacts/logs/rock-authored-matrix-final.log`. Its frozen core test
executable SHA256 is
`9CED12B17F3736D6B20D3218F13C9D5E88F668B0B9FB5FF623C3F9AC7D8E1783`.

The two unchanged successful regression results are retained in
`rock-puzzle-regression.log` and `rock-save-regression.log` (first shared
test executable SHA256
`5E3B3B60C8FC0EA270F76FFAC0E4621185E0227B256DC2E516F329A4A46723BD`).
The separate item-drop/registration matrix was not rerun for this batch.

Installed-data result: the delivered do cue selects frame 28 of 36, duration
2.3333335 seconds. Its Rock translation is
`[74.942825, -371.61758, 1951.3833]`; the full matrix is retained in the test log.
This is measured asset data, not an original-gameplay observation.

Viewer changes compile in the combined release test build (1m53s), logged
in `kq8-artifacts/logs/ordered-loading-shared-viewer-build.log`.
The away/throw/out viewer controller is not exercised by a native or standalone
controller test; the grouped test covers the production core requests, data
frame selection, launch, flight and script recovery.
