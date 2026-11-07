# Installed world patches

The GOG installation includes loose corrections in regional `patch`
folders. The original engine gives these files priority over the
ordinary world files, as documented in
[models-and-textures](behaviour/models-and-textures.md). The Rust
world loader previously read only the archives, bypassing these
corrections.

`GameDir::patch_files` now indexes each patch folder by a
case-insensitive filename. The viewer checks that index before its
archives for both individual reads and script/object enumeration.
A patched name is returned once, and a file found only in the patch
folder is included. A failed patch read is returned as an error by
the lookup rather than replaced with an older archive copy. Other
archive lookup behavior is unchanged.

The script coverage scanner uses the same folder index. It applies
regional patches and the shared `GAME/patch` overrides before
counting scripts and commands. The report contains filenames and
counts, never original script contents.

## Automated verification

Run from the repository root in PowerShell:

```powershell
$env:KQ8_GAME_DIR = (Resolve-Path "..\King's Quest 8 GOG").Path
cargo test --locked -p kq8-viewer --bin kq8-viewer load::campaign_tests::original_world_patches_override_archives -- --ignored --exact --nocapture
cargo test --locked --workspace
cargo test --locked --release -p kq8-viewer --test regions every_region_builds_and_presents_frames -- --ignored --exact --test-threads=1 --nocapture
cargo run --locked --release -p kq8-cli -- script-coverage --out docs/SCRIPT-COVERAGE.md
```

The first check compares all 22 regional patch files with their
selected bytes: Barren 13, Dead City 5, IceWorld 3 and Temple 4 one.
It checks both lookup paths, case-insensitive names and archive
fallback. It then runs the selected Dead City return-portal script
and checks that all four outward portal flags become true.
Before the correction, all 22 selections failed. After it, the
selection and callback check passes. The initial test draft also
had a compile error in its test setup, corrected before the red run.

All 878 standard workspace tests pass. The region check passed in
57.95 seconds: all 12 regions loaded and rendered 30 frames. Local
evidence is `kq8-artifacts/world-patches-{before,after,workspace,regions}.log`
and `kq8-artifacts/region-tests/23456-*`.

The regenerated coverage report reveals two missing commands:
`KQConner::inTrap` and `KQConner::removeFromTrap`, used by Barren's
patched elevator script and IceWorld's patched world script.
Barren's native startup log also records the missing `inTrap` call.
Their exact original semantics are now study question Q201.
Elevator occupancy and return behavior are **not verified**; the
loading pass does not establish their correctness. These commands
remain visibly missing in the generated checklist.

The remaining campaign routes still need verification. No manual
playtest was used, and the original installation was read-only.
