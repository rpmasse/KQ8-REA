# F2 during movement transitions

The separated Q200 study explains the native start-run stall. Accepted F2
re-requests the playing clip, including `stoprun`; it does not replace that
name with `run`. The current transition retains its old model and phase.
When it ends, common completion starts its destination and handles F2's
pending clip request. Connor's ordinary Move state then asks again for the
currently selected input movement, even if Forward never changed.

The Rust walker previously ran that final request only after a reaction.
It now runs after every genuine animation end in ordinary Move, following
common completion. Wait's reaction response, Talk and controller-owned
movement retain their separate rules. Source: `behaviour/camera.md`,
“F2 during movement transitions”, and `behaviour/connor-movement.md`,
Starting and stopping rule 6. No executable or decompiler output was read
by the implementation session.

## Automated evidence, 2026-11-08

The grouped original-data check reads installed `Conner.anm`, `Connerfp.anm`
and their KQS animation data. It passes first attempt, **1/1 in 0.12 s**:

- F2 overwrites a different pending request while retaining the current
  transition's model, phase, destination and destination phase.
- Old `stoprun` finishes, selecting `run` at 0.866667 before the first-person
  `stoprun` replay. Move queues the unchanged held-run request in that same
  completion handling.
- The three-frame replay starts at its authored 0.666667, ends once and
  returns to a running loop instead of holding the last frame.
- F2 while a walking stop transition is only waiting rebuilds the source
  walk and drops the waiting stop request.
- Talk and controller-owned one-shots do not acquire Move's new response.

Six affected existing view-change, reaction and controller-ownership checks
also pass, **6/6 in 0.00 s**. Unaffected movement/campaign tests were not
repeated. The shared core/viewer test build passes in 2m56s.

Automated commands (set `KQ8_GAME_DIR` to the read-only GOG installation):

```powershell
cargo test --locked --release -p kq8-core original_f2_transition_completion_reselects_held_input -- --ignored --nocapture
```

Retained logs in this worktree: `kq8-artifacts/logs/m4-f2-transition.log`,
`m4-f2-affected.log`, and `m4-shared-build.log`.

## Unattended Windows continuation

One short native Daventry run completed all 40 updates using production
executable SHA256
`8BBCCEB30F9C880D458213B3A5494CA4B4454CA4385A285936070041E1F1C52D`.
The queued physical F2 request was accepted at update 11 while the old
`stoprun` still played. At 16 the retained destination phase and Move
follow-up occurred; at 17 the new replay began at 0.666667, ended once and
started `run`. Actual held-run updates 18, 27, 28 and 29 moved 513.1 XY units
in total. The intervening tree collision runs the ordinary crash animation;
the held movement resumes afterward too.

The runner initially reported FAIL because it compared the first and last
run endpoints, which coincide at the same tree. All transition assertions
had passed. The retained log was checked offline using each run update's
displacement from the immediately preceding simulation update, excluding
crash motion: 30.3, 179.7, 179.7 and 123.4 units. This corrected metric passes.
No native rerun or gameplay-code workaround was used. The original FAIL is
preserved alongside `result-offline.json` and `movement.csv` in
`kq8-artifacts/q200-native/1794096839619`. The original Options.cs hash stayed
`C4F0678E45CF42F8DDACB2E09F165BC65B847FB37FDC7E639B664A8CD540AE14`.

The automated run uses an isolated working/save directory and these player
arguments; the local runner is `kq8-artifacts/scratch/q200-native.ps1`:

```text
--game <absolute GOG path> --world Daventry --sound off --hold 1,0,1,5,30 --key physical:f2,8 --updates 40 --frames 1200
```

Neither check establishes simultaneous F2/end ordering, every other
gameplay state, disk-save continuation during the transition, or original
runtime fidelity. No manual testing was performed.
