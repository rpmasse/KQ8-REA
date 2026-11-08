# Controller exit and weapon-list completion

This 2026-11-08 batch follows `behaviour/characters.md`'s forced replacement
table, `conversation.md` rule 15, `inventory.md` rule 8 and
`ranged-weapons.md` rule 8a. It finishes shared behavior families rather
than checking every individual puzzle, item or conversation separately.

Forced Choice now discards the old controller's local continuation and
performs its documented exit effects. Puzzle leaning/camera/music effects
are distinct from successful puzzle completion. Rope displays and launched
projectiles remain independent world objects. Pending swings, shots and
swap assignments are cancelled. Actor physics, held selection, scripted
handsOff and view settings change only where the original exit requires it.

Talk exit is shared by normal completion and forced replacement. The cam0
aim uses the captured partner/talk-spot midpoint plus 1400 vertically.
TalkExit runs before the partner returns to Wait, with goodbye suppression.
Normal finish separately restores panels and proposes Connor's next state;
forced replacement retains the speech queue and panel state. Camera input
reset is ordered before StartChoice's camera commands and suppresses held
camera inputs until release, without changing the camera pose or following.

Explicit inventory refresh and accepted F2 choose by current numeric mode,
view and currently equipped item. A qualified old movement can continue
under the ordinary animation request rules. Pending assignment resolves
equipment again at its actual boundary. Swap movement requests lock input;
an explicit list refresh clears that animation lock, and the next different
swap movement locks it again. The existing saved readying state and pose
lock reconstruct this ownership; no old drawn-item identity field is added.
The loader deduplicates overlapping legacy/live bank requests before model
and material allocation.

## Verification status

The first shared release build failed because the viewer called the core's
private `is_gone` helper. Its existing public `is_deleted` wrapper delegates
to that exact helper; the caller is corrected. Retain
`kq8-artifacts/logs/m4-owner-list-shared-build.log`. The corrected shared
release build passes in **2m41s**, recorded in
`m4-owner-list-shared-build-retry.log`.

The focused core group passes **2/2 in 0.00s**, and the viewer group passes
**7/7 in 1.31s**, with no test failures. Results are in
`m4-owner-list-core.log` and `m4-owner-list-viewer.log`:

- Core: `forced_talk_exit_order_aim_and_queue_matrix`, plus the affected
  existing normal conversation-end check.
- Viewer: `forced_choice_owner_exit_and_reentry_matrix` and
  `forced_choice_camera_input_reset_matrix`.
- Affected viewer matrices: ordinary weapon assignment/checkpoint, shared
  weapon admission, pending request/checkpoint, TempleArmor refresh/restore
  and original-data live bank loading.

Prior core inventory/request checks, cold loading and unrelated native
menu/campaign evidence are retained. No manual gameplay is requested.
Original-size exclusive fullscreen remains a separate host-dependent gate.
Arbitrary reentrant callbacks and later projectile-return consequences are
not established by the current study; see the authored-callback scope in
`m4-completion-audit.md`.

## Reproduce

Set `KQ8_GAME_DIR` to the absolute GOG installation and `KQ8_SAVE_DIR` to
an isolated directory under `kq8-artifacts`. From the worktree:

```powershell
cargo test --locked --release -p kq8-core --lib -- forced_talk_exit_order_aim_and_queue_matrix a_conversation_enters_plays_and_ends_with_talk_exit_before_the_last_talk_complete --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer -- forced_choice_owner_exit_and_reentry_matrix forced_choice_camera_input_reset_matrix original_weapon_assignment_selection_and_checkpoint_matrix weapon_swap_shared_admission_and_script_preparation_matrix weapon_swap_pending_payload_checkpoint_matrix temple_armor_refresh_and_restore_matrix original_temple_armor_live_banks_same_update_matrix --include-ignored --test-threads=1 --nocapture
```

Expect respectively two and seven passing grouped checks. The recorded
run used the same generated release test executables directly after one
shared build. Successful unrelated checks were not repeated. No native
window or manual gameplay test was used for this batch. M4 is approximately
98%; the outstanding identified acceptance gate is supported-display
exclusive fullscreen at the original resolutions.

The required script-coverage report was regenerated successfully after the
final build (`m4-owner-list-coverage.log`, 45.98s build). A read-only display
inventory at 2026-11-08 03:14 UTC still reports remote session 2 and only
3600x2096 at 32 Hz, including raw mode enumeration. The previous inventory
was preserved before capture. No identical native fullscreen failure was
repeated and no display setting was changed.

Subsequent M4 completion: the connected-console production-default fullscreen
route now passes, closing M4 at 100%. See graphics-runtime-completion.md for
its native/frame-submission evidence, retained failures and commands. This
does not change the controller batch's test scope recorded above. M5/M6
remain paused; manual playtesting is still deferred.
