# Swamp and Barren tile controllers

Implementation B, 2026-11-07. This batch implements the original `Tile2`
and `Tile3` controller families, their floor-object classes and the
shared object-projectile cue. It uses the installed GOG data without
modifying it. No original executable, disassembly or `re/` material was
read by this implementation session. Manual play and comparison with
the original executable are **not tested**.

The behavior sources are [secondary-tile-puzzles.md](behaviour/secondary-tile-puzzles.md)
and [object-projectile-cues.md](behaviour/object-projectile-cues.md).
Original-data and material-binding evidence is recorded separately in
[tile-bitmap-rendering.md](tile-bitmap-rendering.md).

## Implemented paths

`secondary_tiles.rs` retains the two different named grids, open outer
boundaries, approach corridors, direction selection from Connor's actual
position, backward choices and Barren's prime sequence. The viewer's
`connor_secondary_tiles.rs` sends the chosen original animation through
the ordinary animation request path. Its steps move Connor through the
existing collision mover; no position interpolation or substitute jump
arc is introduced. Tile2 disables gravity when it requests the jump;
Tile3 waits for actual playback acceptance. The controller owns motion
selection and disables movement lean, while ordinary turning, collision,
physical falling and death remain active. A new Jump while already busy
is deliberately ignored, as directed by the behavior note's exception
for the original reentrant-input defect.

`tile_objects.rs` implements the Swamp power/switch gates, safe scripted
tiles, strict contact cooldown, original single-precision damage factor,
ordered direct-health and ordinary fire-attack events, and one-shot gas
effects. Runtime script assignment and the installed startup assignments
both update the safety classification. Reveal substitutes only the
documented original bitmaps. Texture resolution and quality are unchanged.
The Tile2 Rock action retains controller ownership and restores the
previous projectile after its authored throw.

Barren's object cues retain signed activation counts, first-activation
visibility, held states, original up/down motion selection, phase-preserving
marking and reversal, and stale queued-animation filtering. The controller
records the seven-prime sequence, original sounds, one completion reward,
bridge bitmap commands and delayed lowering of the final prime platform.
Original entry/exit scripts supply the handoffs and camera commands.

Cue 500 launches through the existing projectile simulation before the
ordinary Cue callback. It rotates the local launch offset, retains the
world-relative aim offset, applies the documented damage-override range,
and prepares both the shared projectile script and its impact-animation
recipe. This is shared object behavior, not a special fireball simulation.

`ScriptHost::variables_changed` synchronizes Swamp class gates before
nested callbacks and after assignments returned by host commands. It also
covers an already-populated variable store at script entry and direct
restoration before contact/update.

## Continuation and remaining boundaries

The Rust snapshot retains controller stage, selected destination, motion
pose, prime progress and a pending internal Rock throw. Rebinding does
not replay controller entry/exit effects. The copied grid is rebuilt from
restored object positions before the first busy update. The selected
highlight index survives, but its per-object unlit flag does not: restoring
the same selected cell alone does not relight it.

Object continuation resets Swamp's last-contact timestamp to zero. Lava
retains phase, hold/mark flags, signed count, bitmap and visibility; the
original limitation that the reversed playback speed is not serialized
or reapplied is retained. Reveal override restoration is **recreation
behavior; original-save parity is unverified**.

The shared save system still needs the separately planned frozen-world
clock and absolute-deadline event-queue merge correction. This batch does
not hide that gap with tile-specific queue handling. Full tile save/load
equivalence cannot be claimed before that dependency is fixed. Original
in-flight object-projectile save equivalence also remains unverified;
restoring a queued Cue 500 is not evidence for that separate case.

The existing core Portal command bypasses the viewer's state-change
preflight. Its refusal while Tile2/Tile3 owns Connor, including ordering
after a same-script `tileExit 1`, still needs the shared transition bridge
correction. The established ordinary tile entry and exit callbacks are
covered below; arbitrary Portal requests during tile ownership are not.

The final audit found and corrected a separate recreation bridge gap:
controller outputs wait in `ConnorSim.secondary_calls` until the next
door update. The snapshot now retains this ordered vector independently
of the optional controller, because leaving a controller can produce
cleanup work. Rebinding replaces fresh-entry outputs with the saved
list; the ordinary production dispatcher consumes it once. Older slots
without this field read as an empty list; previously lost outputs cannot
be recovered. Validation accepts only the tile controllers' specific
music commands, tile/door cues, prime sounds, two exit scripts and
projectile-restoration values, with a bounded length and at most one
completion. This is Rust bridge continuation, not a claim about the
original save format or a replacement for the shared clock/queue fix.

## Automated evidence so far

Seven grouped core checks passed across one six-pass run and a targeted
rerun of its corrected floating-point assertion. They cover both grids
and chooser boundaries; prime order/restart/reveal/completion; Swamp
level/clock/power/safe-tile cases; Lava cues/reversal/stale events;
ordinary damage, gas lifetime and class continuation; nested script
assignment and projectile aiming/override boundaries; and the ordinary
animation request waiting/turning path with controller ownership.

Commands:

```powershell
cargo test --locked --release -p kq8-core secondary_tiles_grouped -- --nocapture
cargo test --locked --release -p kq8-core secondary_tiles_grouped_script_order_and_object_projectiles -- --nocapture
cargo test --locked --release -p kq8-core secondary_tile_entry_flags_and_withdrawn_support_matrix -- --nocapture
```

The initial script/projectile assertion compared a rotated floating-point
direction to an exact zero. It failed on a `4.37e-8` residual; the test
now uses a `1e-6` tolerance. No aiming behavior changed for that correction.
After the original-data test discovered missing late safe-script
attachments, the same focused group gained safe→unsafe→safe attachment,
immediate Init, gas suppression and Reveal checks. It passed **1/1**;
the log is `kq8-artifacts/logs/secondary-tile-safe-attachment.log`.

One additional grouped check passed **1/1** for the installed completed
Barren entrance's decimal `move` flag and physical falling with a tile
owner. It accepts unsigned integer spellings with an optional all-zero
decimal tail (`0.`, `1.0`), while rejecting signs, fractions, exponents,
empty and out-of-range values. This narrowly corrects the shared
`kq8-data::world_script::parse_move` path used by the actual patched
`tiletps.cs`; general original numeric-coercion behavior is not inferred.
The physical case removes a 6,000-unit support: ordinary Fall does not
replace the controller, gravity continues, and the real mover still
reaches Die. The retained log is
`kq8-artifacts/logs/secondary-tile-entry-and-fall.log`.

The shared viewer bin-test executable compiled successfully. Its first
compile found two renderer mutable-binding errors and its next compile
found one test-only `Placement::apply`/`to_world` API typo; these were
corrected. The renderer material matrix passed **1/1**. The original-data
matrix's first run correctly failed on 0 safe tiles versus the authored
12, exposing the missing startup attachment path described above.
Later checks exposed static Reveal source labels needing filename
normalization and the installed `0.` move spelling. Both production
issues were corrected; the normalized material matrix passed again.

The corrected original-data matrix passed **1/1** in 2.62 seconds. See
the linked rendering evidence for its full asset/installed-callback
matrix and development failures. Its final log is
`kq8-artifacts/logs/secondary-tile-data-final.log`.

The pending-output snapshot/dispatch matrix passed **1/1** on its first
attempt in 0.02 seconds. It reuses the shared viewer bin-test build
(1 minute 58 seconds) with the live-display batch. It round-trips the
actual snapshot schema and uses the same dispatcher as the door bridge:
completion reward/map/sound and cue ordering, StartExit, music/lowering
after controller exit, projectile restoration and the Swamp door callback
each occur once. It also covers old missing fields and rejection of
invalid operation names, values, duplicates and oversized lists. Synthetic
callback scripts isolate the bridge boundary; the installed callbacks
remain covered by the earlier data and native fixtures. Those successful
checks were not repeated for this serialization-only correction.

Standalone reproduction command:

```powershell
cargo test --locked --release -p kq8-viewer --bin kq8-viewer secondary_tile_pending_effect_snapshot_dispatch_matrix -- --nocapture
```

Actual reuse command:

```powershell
& target/release/deps/kq8_viewer-4aebd42ecd1f894a.exe secondary_tile_pending_effect_snapshot_dispatch_matrix --nocapture
```

The retained result is
`kq8-artifacts/logs/secondary-tile-pending-effects.log`.

## Representative native fixtures

Both fixtures in `tests/secondary_tile_controllers.rs` passed on their
first native attempt. Each uses its own save directory and two fresh
processes; the original `Options.cs` bytes are checked after each run.
They read numerical logs and saved records, without visual inspection.
The fixture health/protection, placements and direct events are isolated
test setup, not evidence of earning those states through the campaign.

```powershell
$env:KQ8_GAME_DIR="C:/Users/Robert/Documents/Codex/KQ8/King's Quest 8 GOG"
cargo test --locked --release -p kq8-viewer --test secondary_tile_controllers -- --ignored --nocapture --test-threads=1
```

- **Swamp: 1/1, 24.32 seconds.** The loaded `fiendoor.cs` Cue13 enters
  Tile2; its delayed Cue14 sets power, position and input state. After
  that authored setup the fixture moves to safe tile02 and starts one
  forward jump. A save at animation phase 0.4 restores the same jump,
  rebuilding all 42 copied positions before busy processing. It lands
  once at update 17, `(52000.094, 96600, 15810)`, on tile12. A Rock click
  starts at 51, its authored throw cue launches the real projectile at 85,
  and impact on tile13 creates the gas at 105. The renderer registers its
  two parts; animation completion deletes it at 117. The subsequent save
  has Tile2 in Keys, handsOff false and its saved projectile restored.
  Removing fixture protection and applying an ordinary lethal attack
  enters Die at 127.
- **Barren: 1/1, 22.84 seconds.** The fixture disables the south entry
  collider as the original On callback does, holds one real platform
  with cues 2403/2400 and uses the installed Cue2 starting position.
  Its save records one activation, held marked `stopup1`, and Connor's
  phase 0.4 forward jump. The fresh process restores that jump and lands
  once at 17, `(92640, 151800.1, 875)`. The ordinary backward clip starts
  at 61 and lands at 102 at `(92640, 149801.31, 545.03)`. The real
  `tiletps2.cs StartExit` runs at 103; the music pop follows through its
  ordinary delayed dispatch. Neither restore replays controller entry.

Native logs:

- `kq8-artifacts/logs/secondary-tile-native-swamp.log`
- `kq8-artifacts/logs/secondary-tile-native-barren.log`
- `kq8-artifacts/secondary-tiles/swamp-1794056879697040900/{save,restore}/kq8-artifacts/logs/`
- `kq8-artifacts/secondary-tiles/barren-1794056914753109700/{save,restore}/kq8-artifacts/logs/`

The production build passed in 108 seconds; the final test-only entry
precondition adjustment rebuilt its harness in 1.88 seconds. The tested
`target/release/kq8-viewer.exe` SHA-256 is
`40159D45AD91A0832D513A20CC400C5C711BF754184A402BC54E957187AA2901`.

These representative fixtures do not complete either entire puzzle or
an earned campaign route. Original runtime and original-save equivalence
remain **not tested**; the shared clock/queue dependency above still
prevents a claim of full continuation parity.
