# Selection at weapon assignment

Study A's `behaviour/inventory.md` rule 12a and `ranged-weapons.md`
rules 6–9 distinguish asking for a weapon from actually assigning its
mode and movement list. The ordinary assignment clears the held item;
an ignored request does not. Internal list changes used by Rope, Tile,
Scales, F2 and ThrowRock do not acquire that side effect.

The viewer records whether a pending assignment came from an ordinary
weapon request. An immediate or delayed assignment emits one notice,
consumed at the production completion boundary. The generic inventory
clear preserves the rope's separate first-person lock. New Rust saves
retain both the pending origin and any unconsumed notice; restoring a
list does not emit another notice. Older Rust saves lack that origin and
retain their previous no-clear behavior. Original save-file parity is
not claimed.

## Verification

The shared core/viewer release build, including the existing conversation
checkpoint target, passes in **4m02s**. The new assignment matrix and the
affected old-slot conversation boundary check pass **2/2 in 0.04s**.
Logs are `kq8-artifacts/logs/m4-weapon-selection-build.log` and
`kq8-artifacts/logs/m4-weapon-selection-viewer.log`.

The grouped matrix uses the production Connor constructor with empty render
assets, original inventory definitions and controlled animation timings.
It covers immediate Fists draw, delayed undraw/away assignment, refused and
no-op requests, F2 during a swap, pending/completed/consumed and legacy
checkpoints, internal list changes, ThrowRock preservation and the existing
interrupted-swap completion boundary. No native or manual test was run.
Successful unrelated checks were retained rather than repeated.

From the worktree, set `KQ8_GAME_DIR` to the absolute GOG installation path
and `KQ8_SAVE_DIR` to an isolated directory under `kq8-artifacts`, then run:

```powershell
cargo test --locked --release -p kq8-viewer --bin kq8-viewer original_weapon_assignment_selection_and_checkpoint_matrix -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer only_the_completed_conversation_away_accepts_a_pending_weapon_list_change -- --nocapture
```

Each command should report a passing check. These checks do not close the
separate script/shared-controller admission gap or the newly confirmed
TempleArmor animation-set stub. Supported-display exclusive fullscreen
also remains unverified. M4 remains approximately 95%.
