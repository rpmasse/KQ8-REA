# Item pickup and pressure-plate release

The Q203 behavior notes distinguish destruction from inventory relocation.
A pickup merged into an existing stack requests deletion of its world
identity. Actual destruction removes that identity from each occupied
trap, issuing Off for the last occupant or Cue 302 otherwise, before its
name and transform become unavailable. A non-merged pickup retains its
identity and resets its transform to the origin; ordinary stay checks
then decide whether it leaves each occupied trap. An origin trap can
retain that identity. Neither case requires a synthetic movement or On.

`ItemObject::pending_delete` is transient and omitted from saves. Pending
items cannot be collected, counted as carried/world inventory, or newly
registered. The item's previous placement remains readable by departure
callbacks. Active's guarded event completion drains further work requested
by those callbacks without recursively destroying the current item.
Script commands, scriptless clicks and chest pickups finish this work.
Carried-item relocation does not enter new traps. Checkpoint restoration
does not dispatch live deletion callbacks; world shutdown has no item
destruction callback path.

## Source

Implementation B used A's prose in `behaviour/traps.md`, “Recoverable Rock,
registration and removal”, and `behaviour/items.md` rules 9/11. A's follow-up
prose also confirms that carry-add resets the surviving carried item's
rotation as well as position, including after a stack merge. B did not
read the original executable, decompilation or disassembly.

## Automated checks

Commands run from the repository root:

```powershell
cargo test --locked -p kq8-core item_removal_tests -- --nocapture
cargo test --locked -p kq8-core -- --skip item_removal_tests
cargo test --locked -p kq8-core active::tests::hidden_and_deleted_items_cant_be_clicked -- --exact --nocapture
cargo test --locked -p kq8-core item_removal_tests::item_destruction_uses_the_spirit_traps_own_departure_handler -- --exact --nocapture
cargo check --locked --release -p kq8-viewer --tests
```

- The first focused build found three test-only type errors: two Vars
  values needed owned strings, and the placement assertion used the data
  type instead of core's transform type. The failure is retained in
  `kq8-artifacts/item-removal-focused.log`.
- After correction, all **six focused cases passed once** in
  `item-removal-focused-built.log`: outer-script ordering and duplicate
  pickup protection; scriptless non-merged pickup; retained origin
  occupancy; Cue 302/Off and sticky behavior with a removed mask;
  callback-triggered deletion without recursion; chest pickup.
- The remaining core run passed **706 cases**, with **17 ignored** and
  the six new cases excluded. One older fixture called the raw interpreter
  and asserted destruction before Active completed the event. The failed
  run remains `item-removal-core-regression.log`.
- That fixture now checks the pending, uncollectable state, then completes
  the event and checks Gone. Only that failing case was rerun; it passes
  in `item-removal-deleted-click-boundary.log`. The 706 successful cases
  and the six focused cases were not repeated.
- Final review preserves the spirit subclass's own departure handler,
  rather than also calling a generic trap script, and limits the known
  carried-origin transform to Connor. The seventh focused case passes
  once in `item-removal-spirit-review.log`. The successful native case
  preceded this review and is not repeated; its Rock/ordinary-trap path
  is unchanged. The viewer save fixture includes the transient field.
- Final release viewer/test-target compilation passes in **19.71 seconds**,
  recorded in `item-removal-viewer-tests-check.log`.

## Earned campaign check

The new route uses the genuine WesternPlateRockReload save, picks its
actual world Rock through the normal visible click, and reloads in a
fresh Windows process. It requires two carried Rocks afterward, no
remaining world Rock occupant, both portal masks zero, unchanged other
inventory/stats/quest flags, and unchanged original Options.cs. The
successful earlier Rock-hold test is not repeated.

```powershell
$env:KQ8_GAME_DIR=(Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
$env:KQ8_WESTERN_PLATE_ROCK_SLOT=(Resolve-Path -LiteralPath 'kq8-artifacts/earned-western-plate-rock/1794039587261976800/slots/slot-3.json').Path
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::picking_up_earned_rock_releases_western_plate_and_reloads -- --ignored --exact --nocapture
```

The route **passes once in 28.75 seconds**, recorded in
`kq8-artifacts/western-plate-rock-pickup.log`. The genuine output is
`kq8-artifacts/earned-western-plate-rock-pickup/1794042294163512500/slots`:
slot-2 is WesternPlateRockPickup; slot-3 is WesternPlateRockPickupReload.
The prior held-Rock checkpoint remains available for portal traversal.

## Remaining fidelity work

Q203 is still partial. Last-unit Rock identity, non-Rock stack removal and
activation, full saved-name contact reattachment, other registration
paths, and collision-type/mask overrides remain. The loaded trap-list
ordering is not proof of the original spatial traversal (Q205). Original
aim/arc remains Q202. This release work does not prove portal traversal,
the remaining Dead City quests, or the later campaign. Manual testing
and original-engine runtime comparison are **not tested**.
