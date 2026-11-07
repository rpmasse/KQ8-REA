# Scripted prop placement and occupied triggers

The original rule is in `behaviour/traps.md`, rule 5(b), Q102. A script's
instant move does not sweep its path or enter a trigger at its destination.
After the outer script and its nested calls finish, an already occupied
trigger checks the object's current shape and position. It sends Off when
the final occupant leaves, or Cue 302 when another occupant remains. A
removed collision mask does not prevent this departure.

`active.rs` now retains the installed prop collider metadata, updating it
on physical pushes. Script moves update logical and saved prop placement
immediately. The outer script-call boundary drains moved prop names using
their final placement. This preserves an out-and-back move within a single
script and lets departure callbacks see the script's final variables.
The per-trigger check refreshes placement and deletion state because an
earlier departure callback may move or delete the same object.

The viewer registers loaded colliders, including restored props. New runtime
copies acquire their shape when physically pushed. Callback requests created
while the viewer applies an earlier request are processed immediately in
their original order, before returning to the remaining requests.

The original-data check exposed a loader omission: saved prop registration
accepted only `KQObject`, although the loader also represents `KQBoulder`
as a still physical prop. `KQBoulder` now uses the existing prop placement
capture and restore path. Its unstudied animation/physics are unchanged.

## Automated evidence

Run from the repository root in PowerShell. These checks have already been
run as recorded below; they are reproduction commands, not a request for
manual playtesting or repeated successful runs.

```powershell
cargo test --locked -p kq8-core prop_placement_tests -- --nocapture
cargo test --locked -p kq8-core --lib -- --skip prop_placement_tests
$env:KQ8_GAME_DIR = (Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
cargo test --locked --release -p kq8-viewer --bin kq8-viewer load::campaign_tests::original_gnome_boulder_script_leaves_its_disabled_trigger_before_returning -- --ignored --exact --nocapture
```

The first eight focused tests pass once in
`kq8-artifacts/prop-placement-focused-fixed.log`. They cover immediate
logical position, nested scripts, request order, mask removal, no new
entry along the jump or at its destination, multiple occupants, chained
departures, oriented boxes, ball stacks and deletion notices.

The remaining core suite passes once: **686 passed, 17 ignored**, excluding
those eight tests (`prop-placement-core-regression.log`). This run precedes
the review correction below; it was not repeated after passing.

Review identified stale position/deletion state between overlapping
triggers. The dispatcher now refreshes that state for each notification.
Both new overlap cases pass once (`prop-placement-overlap.log`). A separate
JSON checkpoint test rebuilds collider metadata and leaves a restored,
disabled trigger without another physical push; it passes once in
`prop-placement-restore-overlap.log`. That command's skip filters selected
only the restoration case; the two overlap cases ran separately. There
are **11 passing focused cases**, each run successfully once.

The original Gnome check loads the installed level, collider, ground and
`bouldtrp.cs`. An accepted physical push enters the actual trigger. The
check requires its original On script to clear the mask, move the boulder,
leave the trigger in the same update, preserve the camera/Connor requests
and schedule both original cues. It then serializes saved prop positions
and loads the original Gnome data again through `Arrival`, requiring the
model, collider, saved placement and Dropped flag to agree, preserving
the original object's click-target presence.
It isolates the push by omitting other scenery obstacles. The complete
check passes once in **2.30 seconds**, including the fresh data load
(`gnome-boulder-placement-persistence-fixed.log`). The release viewer test
target builds successfully. No successful original-data run was repeated.

## Retained failures and limits

The first focused build failed because a test passed a string slice where
the variable API requires an owned string (`prop-placement-focused.log`).
The test was corrected before any case ran. The first Gnome check failed
after 3.47 seconds because its expected name was lowercase; the original
loaded name is `Bouldtrp`. Expectations now use the loaded name, without
changing the original script or engine behavior
(`gnome-boulder-placement-original.log`).

The second original-data run passes the entry, mask, departure and current
logical position assertions, then fails after 2.81 seconds because there
is no saved `boulder` entry (`gnome-boulder-placement-original-fixed.log`).
The failure was recorded and the fixture was stopped for loader inspection.
The source's class filter identified the actual persistence omission;
the test's save requirement was retained and expanded to a fresh data load.

That next check passes the restored collider, model and saved placement,
then fails at 2.37 seconds because the fixture assumes a click target.
The installed KQBoulder has none. The expectation now compares target
presence with the original loaded object instead of requiring one
(`gnome-boulder-placement-persistence.log`).

This does not complete Gnome's boulder cutscene or any campaign route.
Full animated-object entry coverage, exact interruption of an ongoing
physical collision pass, Connor's broader placement scheduling and drawn
prop rotation remain incomplete. The unknown Rock projectile/item handoff
and deletion/pickup occupancy rules remain Q203. The existing asymmetric
box-centre rule remains Q204. No manual or visual test was performed.
The viewer callback-request drain builds with the release test target;
its live viewer execution is not separately tested in this step. The
Gnome reload is a new loader instance, not a fresh Windows process.

Implementation uses existing Rust code, the prose behavior note and original
data read from the installed game. Implementation session B did not read
the original executable or decompiled code.
