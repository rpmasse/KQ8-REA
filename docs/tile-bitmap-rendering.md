# Tile bitmap rendering

Implementation B, 2026-11-07. This is the narrow rendering adapter for
the shared Swamp/Lava controller work, not a completed M5 or M6 gate.

## Sources and scope

[Secondary tile puzzles](behaviour/secondary-tile-puzzles.md), rule 23,
supplies the two simultaneous Swamp Reveal substitutions and ordinary
override restoration. Its Lava class rules 36–40 supply the marked top
bitmap selected by Cue 2400. Class decisions remain in
`WorldState::tile_bitmap_pairs`; rendering reads their result.
[Characters](behaviour/characters.md), rule 4a, supplies ordinary
`changeBmp` behavior. The implementation uses those prose notes and
existing Rust rendering paths. It does not inspect Mask.exe, Ghidra,
decompiled output or `re/`.

`anim_view.rs` now applies replacement sets against the original texture
names. Normal object overrides come first; class pairs take precedence
for their own named targets. An unrelated normal override remains active.
This is a bounded combination rule: the study specifies restoration of
normal overrides and does not establish discarding unrelated targets;
known tile scripts do not exercise that conflict.

Loaded static `Look` names include their source location, such as
`vol:Swamp/8bit/GASTRAP.BMP`. The static binding keeps the filename,
matching the existing animated path and the script/class pair names.
The material matrix uses both archive and Windows-style source labels.

Animated objects retain their existing per-instance materials. Aliased
animation slots save each original material once. Later opacity copies
inherit the saved original texture. Static Swamp parts retain their
normal material handle and get a private material only when their first
applicable replacement occurs. Other copies of the same model retain
their textures. Changing a bitmap refreshes the drawn material binding
so the existing full-bright highlight adapter can follow it; the normal
lighting, opacity and haze settings remain on the normal material.

`terrain_view.rs` only records original texture names and attaches this
binding to static KQSwampTile parts. Geometry, animation timing and the
existing shared model/image caches are unchanged. Replacement images use
the existing `swap_looks` and live world-palette path, with original asset
dimensions and quality. Missing replacement images leave the original
texture. Repeated unchanged replacement sets do not allocate materials.

## Automated checks

From the repository directory:

```powershell
cargo test --locked --release -p kq8-viewer --bin kq8-viewer tile_bitmap_material_matrix -- --nocapture
```

The grouped, headless material matrix checks both two-part Swamp Reveal
families, all eight Lava bitmap choices including the original-name
identity choice, shared-model copy independence, unchanged-repeat
behavior, ordinary override fallback, restoration, aliased animation
slots, missing replacement images and later opacity copies. It also
checks the normal-handle handoff after a synthetic highlight copy and
preserved lighting/opacity/haze values. Expected result: one passing test
and a `tile bitmap matrix:` summary line.

Result: the expanded source-label matrix **passed**, 1/1 in 0.00 seconds
after compilation completed in 72 seconds. The canonical Cargo command
above produced `kq8-artifacts/logs/tile-bitmap-material-normalized.log`.
The earlier bare-filename matrix also passed; that first check reused
the shared compiled test executable directly:

```powershell
& '.\target\release\deps\kq8_viewer-4aebd42ecd1f894a.exe' tile_bitmap_material_matrix --nocapture
```

Output is retained in
`kq8-artifacts/logs/tile-bitmap-material-matrix.log`. No independent native
game run is requested for this adapter. The combined tile fixture will
provide the next integration evidence.

The first shared `cargo check --locked --release -p kq8-viewer --bin kq8-viewer` reported
two E0596 errors in this adapter: the values returned by
`Assets::get_mut` needed mutable bindings. Both bindings were corrected.
That initial output was session 35825 and was not retained as a separate
log file. The corrected shared bin-test compilation passed as above.

The first shared viewer bin-test compilation then stopped before running
any tests with E0599 in the new data matrix's diagnostic placement
report: it used `apply` on a core `Placement`. The existing method is
`to_world`; the test was corrected to use it. Production behavior was
unaffected. Session 6925 records that failed compilation.

## Original-data integration matrix

The separate `secondary_tile_data_tests.rs` module uses the normal
`load_ground` / `load_objects` path against the read-only GOG install.
Its sources are secondary-tile-puzzles.md rules 9, 16, 23, 26, 34, 36
and 43, object-projectile-cues.md rules 7–10, and the installed readable
data and loose patch scripts. This is distinct from a comparison with
the running original game.

```powershell
$env:KQ8_GAME_DIR = "C:\Users\Robert\Documents\Codex\KQ8\King's Quest 8 GOG"
cargo test --locked --release -p kq8-viewer --bin kq8-viewer original_secondary_tile_data_matrix -- --ignored --nocapture
```

One grouped check covers:

- 42 loaded Swamp tile records, including 12 scripted safe tiles;
  their authored grid placements and both original material-name pairs;
  four indexed Reveal images and gas/impact animation recipes.
- Six loaded totem firing configurations, the original fireball speed,
  range and Fire kind, its prepared shape, and the installed shared
  impact script.
- 35 loaded Lava platform records, their initial hidden placement,
  eight authored movements and all eight indexed top-bitmap recipes.
- Exact installed Barren patch selection; script execution producing
  the 32 original staggered Cue 2403 deadlines and the three documented
  omissions; initial south, completed south and north entry callbacks;
  both patched exit destinations.

The script checks invoke callbacks directly and inspect requests/events;
they do not claim that Connor traversed the route or that timed event
delivery, controller physics, projectile impact or a saved game completed.
The test prints loaded placement and collision bounds for the planned
native fixture's Swamp tile03/tile13 and Barren tile03; these observations
do not replace that fixture. Expected result: one passing ignored test
with separate Swamp and Barren summary lines.

The first run **failed** in 2.33 seconds: all 42 Swamp tile records were
present, but zero had their safe-script flag set instead of the expected
12. This is a loader integration defect; the assertion was retained.
The rest of the Swamp assertions and Barren checks were not reached.
Output is retained in
`kq8-artifacts/logs/secondary-tile-data-matrix-first.log`. The exact run
used the same environment variable above and the shared executable:

```powershell
& '.\target\release\deps\kq8_viewer-4aebd42ecd1f894a.exe' original_secondary_tile_data_matrix --ignored --nocapture
```

The loader correction now updates the class flag and static callback
registration when the startup script attaches `fiensafe.cs`; the matrix
also checks that callback for every safe tile. The next Cargo attempt
stopped before running tests on E0502 in the independently edited route
driver: an immutable landmark borrow overlapped a `wait_started` update.
That output remains in `kq8-artifacts/logs/secondary-tile-data-retry.log`.
After the route correction, compilation passed in 68 seconds. The data
run then passed the 42/12 flags and callback checks but failed in 1.43
seconds at the first source-texture assertion: the loaded name was
`vol:Swamp/8bit/GASTRAP.BMP`, not a bare filename. This exposed a real
omission in the new static binding, which now strips the source location
as the existing animated path does. The data assertion compares original
filenames, and the affected material matrix now exercises qualified
labels. That failure is retained in
`kq8-artifacts/logs/secondary-tile-data-corrected.log`; the next corrected
run reused the newly compiled executable after the expanded material
matrix passed.

That next data run passed all Swamp checks and the Barren metadata,
authored motions, pictures and staggered-start checks, then **failed** at
an entry `Move` assertion in 5.07 seconds. Its output remains in
`kq8-artifacts/logs/secondary-tile-data-normalized.log`. Source diagnosis
found the installed completed-south line uses move flag `0.`, whereas
the shared `parse_move` accepted only Rust integer-token syntax. This
is a shared command-parser gap, not a reason to change the installed
patch or weaken its expected destination. After the narrow parser
correction, the complete original-data matrix **passed**, 1/1 in 2.62
seconds, following a 120-second dependency/viewer rebuild. The exact
Cargo command shown above produced
`kq8-artifacts/logs/secondary-tile-data-final.log`. This verifies all
three entry callbacks and both exit destinations. The already passing
material matrix was not repeated for the parser change.

The loaded collision reports before that failure place Swamp tile03 at
(50,000, 98,600, 15,045), with local bounds (−560, −560, 0) to
(570, 570, 1,250) and world top 16,295. Adjacent tile13 is at
(52,000, 98,600, 15,045), bounds (−570, −610, 0) to (550, 560, 740),
and world top 15,785. Both retain authored collision type 16,417 and
mask 5. The next run also reported tile02 at (50,000, 96,600, 15,045),
local bounds (−590, −590, −1,210) to (560, 570, 990), world top 16,035;
tile12 is at (52,000, 96,600, 15,045), with the same bounds and world top
as tile13. Barren tile03 loads at (92,640, 151,943, −700), bounds
(−500, −500, 0) to (500, 500, 750), world top 50; its initial hidden
platform retains collision type 65 and mask 0. This is its loaded
position before the authored rise, not a claim about its moving support
surface.

## Native fixture preparation, read-only

No additional game run was made. Existing loader logs establish that
Barren's active `tiletps` entrance box contains the synthetic original
south start point. The original On script removes its Connor collision
before the Cue2 placement; a representative fixture must preserve that
precondition to avoid a second entry, extra activation cues and an
automatic walk overriding its setup.

The successful loader check found no Trap or scripted building with
`tiletrap.cs`. A further read-only inspection used the existing command:

```powershell
.\target\release\kq8.exe text vol:Swamp/kq/fiendoor.cs --game "C:\Users\Robert\Documents\Codex\KQ8\King's Quest 8 GOG"
```

The same command read `tiletrap.cs`, `newWorld.cs` and `swdoor.cs`.
An attempted `tiletrap.kq` read reported no such archive entry, and
`newWorld.cs` does not load one. This does not prove the script is never
referenced anywhere; it rejects inventing that contact trigger for this
loaded-world fixture. The loaded `fiendoor` supplies an authored entry
path: Cue 13 requests Tile2 and schedules Cue 14 after 1.2 seconds;
Cue 14 places Connor at the western approach and enables inTileState
and PowerOn. A fixture using those callbacks must wait for the delayed
placement before setting up its representative jump. The native
controller evidence belongs in [the controller report](secondary-tile-controllers.md).

The final local evidence directory is
`C:/Users/Robert/Documents/Codex/KQ8/KQ8-REA/kq8-artifacts/logs/`.
The two final passing logs are `tile-bitmap-material-normalized.log`
and `secondary-tile-data-final.log`. Earlier failures remain alongside
them under the filenames recorded above.

The material matrix uses in-memory material/image handles; it does not verify GPU
pixels, original-runtime presentation or a full campaign. Those remain
**not tested** by this adapter. Existing palette checks are reused rather
than repeated. No manual testing is included.
