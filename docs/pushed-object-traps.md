# Pushed objects and puzzle triggers

Physical pushes now notify traps using the moved object's identity,
live collision type and original shape. Previously the viewer moved
the drawing, collision and saved placement but sent no trigger event.
This matters to the original Dead City girltrap, Temple2 booktrap and
Gnome bouldtrp scripts, which explicitly name their pushed objects.

Ball stacks follow the accepted contact/slide segments. Boxes retain
their orientation and are swept through translation; they are not
replaced by balls or enlarged world bounding boxes. A cancelled uphill
push has no accepted movement and enters nothing. The trap's mask
controls entry; its existing On/Cue/Off dispatcher handles occupancy.

Before a callback, the logical prop position and saved placement are
updated. Traps now expose their resolved position to getLoc and
checkDistance, as those original callbacks require. A moved prop that
has no clickable box can also resolve its recorded position.

## Sources and limits

Rules come from `behaviour/traps.md` rules 4-9 and
`behaviour/pushing.md` rule 4. The installed original scripts and
object data are read at runtime. No game assets or extracted scripts
are stored in the repository.

The old logical-centre stay test remains unchanged. Its treatment of
an asymmetric rotated box is not established by the notes; Q204 asks
for that study. The new box collision is oriented, but its centre rule
is not claimed to match the original exactly.

The viewer dispatches these callbacks in its existing push-application
stage, after Connor's collision update and before applying the next
queued push. Exact interruption inside his collision pass remains a
fidelity gap. Script placement/removal of an occupant, pickup/deletion
release and the Rock projectile handoff remain separate work. This
change does not complete those puzzles or M6.

## Automated checks

```powershell
cargo test --locked -p kq8-core prop_traps -- --nocapture
```

Eight synthetic cases cover rotated/thin boxes, the preserved centre
rule, a ball-stack wall slide, masks, rejected uphill movement,
position-dependent callbacks and multi-occupant leave cues. The initial
seven pass. Review then removes an unjustified transformed-centre
assumption; the revised centre case passes separately. The other six
successes are not repeated. The remaining core checks pass once:
678 passed, 17 ignored, the seven then-existing focused cases excluded.
Review adds one case for a callback changing the next object's type
before its queued collider updates. The dispatcher uses the live type;
that eighth case passes once. The final viewer release check passes.
Logs: prop-trap-live-type.log and prop-trap-final-viewer-check.log.

```powershell
cargo test --locked -p kq8-core -- --skip prop_traps
$env:KQ8_GAME_DIR = (Resolve-Path "..\King's Quest 8 GOG").Path
cargo test --locked --release -p kq8-viewer --bin kq8-viewer load::campaign_tests::original_temple2_pushed_book_enters_its_trigger_and_starts_the_hall_sequence -- --ignored --exact --nocapture
```

The Temple2 check reads the installed book collider, trap, script,
ground and controller animations. It uses one synthetic physical
push, with other obstacles omitted, to isolate the real callback:
expected On, trap disabled, book deleted, camera/control requests,
controller created and Connor's first scripted movement requested.
It is not a playable route or a complete hall-sequence check.

This installed-data check **passes in 1.74 seconds**. It runs once
after correcting the module filter; no successful asset replay is
repeated. No manual testing or visual inspection is used.

Evidence logs are under kq8-artifacts: prop-traps-focused.log,
prop-traps-core-regression.log, prop-trap-centre-preserved-fixed.log
and temple2-book-trigger-original.log. Earlier diagnostics are also
retained: temple2-book-trigger-native.log (private-access compile
error), temple2-book-trigger-native-fixed.log (wrong module filter,
zero tests), prop-trap-centre-preserved.log (array-size inference
compile error). No zero-test run is counted as a passing check.
