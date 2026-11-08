# Weapon requests and inventory animation changes

The 2026-11-08 batch implements the shared weapon-request rules from
`behaviour/ranged-weapons.md` rules 2–9 and the common state ordering in
`behaviour/conversation.md`. Script preparation, controller admission and
actual assignment are separate steps. A waiting request retains its mode,
optional movement, modifier and origin without prematurely changing the
held item or movement list. Raw numeric modes retain their exact value and
current list. Required-mode completion runs its callback before rereading
the requirement; correction stays in the same Swap controller.

The shared pending checkpoint retains those payloads and the forced Choice
target. This batch establishes Choice admission and return, but does not
yet remove every dedicated old controller on forced replacement. The
[subsequent controller batch](controller-exit-completion.md) implements and
checks that exit work from `characters.md`.
Arbitrary reentrant callbacks are not established by the current study.

TempleArmor pickup and `KQConner::helmetize` now change the actual equipped
third-person mappings and refresh the current list without an ordinary
weapon assignment. The changes survive removal, Rust saves and world
travel. First-person mappings stay unchanged. The loader prepares changed
banks and retained old banks before continuation. See
[mapping details](temple-armor-mappings.md).

## Verified results

The shared release build passes in **4m04s**. The core group initially
passed **7/8**; its TempleArmor fixture incorrectly supplied an item
instance to `dropItem`, which takes an item type. Changing that test command
to `KQMonster::dropItem Connor TempleArmor 7 8 9 0 0 0` made the focused
retry pass **1/1 in 0.01s**, after a **1m11s** core rebuild. Production code
was unchanged by the correction. Retain both the failure and retry logs.
The seven successful core checks were not repeated.

The viewer group passed **9/9 in 5.70s**. It covers script preparation,
lower admission, optional movements, raw modes, required-mode correction
and callback rereading, forced Choice return, full pending payload saves,
ordinary assignment timing, conversation continuation, TempleArmor refresh
and actual original-data bank loading. Required-mode checks include
draw/undraw bypass, an internal away/assignment/out correction with pending
Talk, and a callback changing the required numeric mode.

Core checks also cover lower request gates, existing key/slot/script
behavior, conversation admission/completion, and actual equipped-type map
changes, repeated changes, removal and Journey serialization. These are
grouped component checks, not proof of complete campaign fidelity.

Logs under `kq8-artifacts/logs/`:

- `m4-weapon-armor-build.log`
- `m4-weapon-armor-core.log` (including the original fixture failure)
- `m4-armor-core-retry.log`
- `m4-weapon-armor-viewer.log`

No native or manual gameplay test was run. Previously successful unrelated
checks remain valid and were not repeated. Original save-file equivalence,
arbitrary reentrant callbacks and supported-display exclusive fullscreen
remain open. Forced replacement cleanup and the post-review list-selection,
input-lock and duplicate-load corrections pass the subsequent grouped batch.

## Reproduce the grouped checks

From this worktree, set `KQ8_GAME_DIR` to the absolute original installation
path and `KQ8_SAVE_DIR` to an isolated directory under `kq8-artifacts`.
Each command below should report one passing grouped matrix:

```powershell
cargo test --locked --release -p kq8-core weapon_request_lower_gate_matrix -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-core temple_armor_mapping_persistence_matrix -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer weapon_swap_shared_admission_and_script_preparation_matrix -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer weapon_swap_pending_payload_checkpoint_matrix -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer temple_armor_refresh_and_restore_matrix -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer original_temple_armor_live_banks_same_update_matrix -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer original_temple_armor_banks_load_matrix -- --include-ignored --test-threads=1 --nocapture
```
