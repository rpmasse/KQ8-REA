# Scripted character preparation and health

This implements `KQMonster::preSpecial` and `KQConner::reduceHealth`
from study session A's
[scripted-controller-commands.md](behaviour/scripted-controller-commands.md),
rules 1–4. Implementation session B used that prose, existing Rust and
read-only original script data, never executable or disassembly material.

`preSpecial` accepts an exact-case monster name, including Connor. It makes
current playback interruptible and remembers Stop separately from actual
and pending animation names. It does not play Stop, reset phase, change
the controller, alter hands-off, move the actor or emit completion. Ordinary
requests still consider authored transitions. New playback resets the
interruption override, and a later interruptable cue may replace it.
The original animation still supplies its eventual EndLoop name.

`reduceHealth` samples current level and health, uses ten health points per
level, and computes a whole-point delta. The reduction retains decimal
double precision; the minimum first rounds to single precision. The delta
is queued as a normal health-only stats request due now. It is not an
attack. Repeated calls before delivery sample the same health, and ordinary
stats handling retains its health cap, level processing and strictly
negative death test. The command does not rewrite CurRelDamage/CurRelHealth.

The grouped checks cover both commands' argument counts, exact names,
ordinary monster and Connor preparation, retained phase/owner/input,
authored transitions, later cues and completion identity. A shared health
matrix covers the shipped `.5 .02`, low-level zero, healing minimum,
unclamped fractions, malformed numeric text, single-precision rounding,
queue timing, repeated deltas, immunity/protection bypass, dying/rope stats,
maximum cap and zero-XP level processing.

One original-data callback matrix executes Swamp `Swhnchmn.cs` Die and
Temple4 `altar.cs` Cue 1/7. These are representative consumers of the shared
mechanisms, not separate implementations for each enemy or altar scene.
The Henchman callback must preserve preparation → move → Special kill2
request order. The altar callback must preserve 430 health until queued
delivery, then produce 215 health at level 43.

## Validation

The grouped core run passed **4/4** checks on 2026-11-07 (59.16 seconds
for the release build, 0.02 seconds for the tests), including the two
original callbacks. No successful case was repeated. The viewer release
compile check also passed in 17.61 seconds; it did not rebuild or replace
the executable used by the separate earned campaign run.

```powershell
$env:KQ8_GAME_DIR = (Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
cargo test --locked --release -p kq8-core scripted_character_grouped -- --include-ignored --nocapture
cargo check --locked --release -p kq8-viewer
```

Evidence: `kq8-artifacts/scripted-character-core-corrected.log` and
`kq8-artifacts/scripted-character-viewer-check.log`.
The initial compile failed because the helper accessed a private deleted-
object set. It was corrected to use the existing `is_gone` accessor;
that initial diagnostic remains in `kq8-artifacts/scripted-character-core.log`.
No gameplay test failed in that attempt because compilation had not finished.

No original-executable runtime comparison, native viewer fixture or manual
testing has been performed for this batch. Transient preparation save/reload
is **not tested**; the new live flags are not persisted by this batch. No
original-save equivalence is claimed. Numeric object-identifier resolution
is still unavailable because Rust has no original object-ID registry; the
command logs this limitation rather than inventing vector-index IDs.
Non-finite numeric input, overflow and zero-level Connor are outside A's
supported domain and produce a diagnostic with no queued adjustment.

Trap occupancy, saved trap-name restoration and inline world loading are
a separate Q201 implementation, not part of these two commands.
