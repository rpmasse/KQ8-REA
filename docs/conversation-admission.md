# Conversation state admission

The viewer now submits ordinary Connor state requests through one replaceable
pending slot. It uses Study A's `behaviour/conversation.md` admission addenda,
`azriel-hammer.md` catch rules and `characters.md` Choice/Wait rules. Request
payloads are preserved, rather than retrying an unentered conversation on
every update.

The start message clears speech, captures the partner's current position and
prepares Stop once. State admission is separate: the current owner decides
whether the older pending request or the new request can enter. A successful
start-message request hides the panels when handsOff is clear, including when
an older strong request wins and Talk remains pending. Talk's camera and
approach setup run only when Talk actually enters.

Completion hooks arbitrate synchronously at Jump, Fall/landing, Special,
weapon-change and hammer-catch boundaries. A completed standing movement
retries the slot only after an eligible non-Stop/non-gesture end successfully
requests Stop. An obsolete Talk exit cannot replace a newer controller.
Choice clears the slot immediately before marking the choice made and issuing
DoChoice. An admitted scripted death cancels the remaining movement passes
without inventing a new Mover state.

Rust saves carry a typed pending request, admission gates and, when needed,
the airborne controller pose. Existing Talk, Special and weapon snapshots
retain their own poses. A missing field in an older Asked save schedules one
start message; a present empty slot does not resurrect a replaced request.
This preserves the recreation's own continuation and does not establish
original save-event queue equivalence.

## Verification status

The shared release build passes in 3m56s. All three production dispatch,
pending-continuation and legacy-migration viewer matrices pass in the
11-check viewer group. Core checks pass 19/20, including the admission gate,
conversation and existing movement regressions. The new completion matrix
failed because its skipped-flight fixture retained the constructor's grounded
contact flag. Both synthetic airborne setups are corrected; production code
and assertions are unchanged. Its focused retry passes in
`m4-selection-and-phase-core.log` after a 1m12s core build. All 20 selected
core checks now pass across the initial run and this one-check retry; the
19 successful checks were not repeated.

Logs: `kq8-artifacts/logs/m4-final-admission-graphics-build.log`,
`m4-final-admission-core.log` and `m4-final-admission-graphics-viewer.log`.

The grouped checks cover the gate/priority matrix, start preparation,
completion phases, production request dispatch, payloads, pending snapshot
continuation and legacy migration. The affected existing checks cover
conversation approach/exit, F2 transition recovery, reaction completion,
weapon swap, landing, Special and death ownership. These checks are component
evidence, not a complete campaign run or original runtime comparison.

## Commands

Set `KQ8_GAME_DIR` to the absolute read-only original installation and
`KQ8_SAVE_DIR` to an isolated ignored directory. Compile once:

```powershell
cargo test --locked --release -p kq8-core -p kq8-viewer --no-run --lib --bins --test conversation_checkpoint --test live_display
```

The shared executable run combines the following filters so overlapping core
tests execute once:

```powershell
cargo test --locked --release -p kq8-core conversation_admission_grouped -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-core conversation::tests:: -- --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer conversation_admission_grouped -- --include-ignored --test-threads=1 --nocapture
```

After validation, each selected check should report success. The external
conversation checkpoint target is compiled for compatibility; its previously
successful native routes are retained unless new evidence requires a rerun.
