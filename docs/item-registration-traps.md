# Fresh item registration and the earned Rock plate

The earned Dead City Rock now holds the western pressure plate after
landing and after a fresh Windows process reload. The previously failing
route passes once in **46.00 seconds**. Both portal masks are 4 and the
plate retains the actual world Rock identity, `NoName98`, in both saves.

The separate original-engine study answered Q203 in
[traps.md](behaviour/traps.md#recoverable-rock-registration-and-removal-q203-2026-11-07)
and [items.md](behaviour/items.md). The projectile itself cannot occupy
this trap because both collision details are 1. Its Die script creates
a fresh recoverable item whose active collision registration checks
stationary overlaps. This is different from a script moving an already
registered object, which does not enter a destination trap.

## Implementation

`script.rs` captures command-owned continuation tokens before entering
nested scripts. `active.rs` resumes each item's registration after each
callback, reading current placement, deletion state and trap masks. The
outer command retains its own final NewInvItem result even if a callback
creates another item. The multiple-Rock drop sets NewInvItem to the
carried identity first and to the new identity only after registration.

`trap.rs` rotates the BOUNDS-derived local ball centres by the item's
full orientation without changing their radius. The normal trigger
geometry and mask checks determine entry; no forced On or synthetic
flight is used. Item location queries and script moves now use the live
item transform. Existing-item script moves run only occupied-trap stay
checks after the outer event.

Registration is connected to runtime world-inventory creation, fresh
items made by the existing drop path and chest contents. Opening a chest
finishes its state/remembered-item changes before completing the click's
pending movement notices and deleted-name bookkeeping. The viewer's
diagnostic command path now uses the complete console command boundary.

## Commands and results

Run from the repository root in PowerShell. These are reproduction
commands; the successful checks listed here were not repeated.

```powershell
cargo test --locked -p kq8-core item_registration_tests -- --nocapture
cargo test --locked -p kq8-core active::tests::chest_item_registration_finishes_its_click_before_pending_departures -- --exact --nocapture
cargo test --locked -p kq8-core --lib -- --skip item_registration_tests
$env:KQ8_GAME_DIR = (Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
$env:KQ8_WESTERN_PLATE_RELEASED_SLOT = (Resolve-Path -LiteralPath 'kq8-artifacts/earned-western-plate-release/1794034374631528600/slots/slot-3.json').Path
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::earned_rock_holds_western_portal_open_after_reload -- --ignored --exact --nocapture
cargo check --locked --release -p kq8-viewer
```

- Eight focused registration cases pass in `item-registration-focused.log`.
  The ninth initially fails only because its count assertion uses the alias
  Rock where `Items::count` requires INVITEM_Rock. Correcting that assertion
  and rerunning only that case passes (`item-registration-nested-fixed.log`).
- The remaining core suite passes once: **697 passed, 17 ignored**, with
  the nine registration cases excluded (`item-registration-core-regression.log`).
- Review finds that chest callbacks need their enclosing click boundary.
  The correction's new case passes once (`item-registration-chest-boundary.log`).
  This case was added after the broader suite; that suite was not repeated.
- The earned Rock native route passes once in `western-plate-rock-registration.log`.
  It uses ordinary inventory selection and the previously established visible
  ground click, not a new inventory grant or forced trap callback. It verifies
  the carried count changes from two to one, other inventory/stats/flags are
  unchanged, the world item occupies the plate without Connor, and a fresh
  process retains the same item and both enabled portal masks.
- The final release viewer check passes in
  `item-registration-final-viewer-check.log`. The successful campaign route
  preceded the chest-only correction and is not repeated.

The genuine checkpoints are under
`kq8-artifacts/earned-western-plate-rock/1794039587261976800/slots`:
slot-2 is WesternPlateRock; slot-3 is WesternPlateRockReload. The original
failed run, `1794035062532059200`, and `western-plate-rock-native.log`
remain intact. No original game files were modified.

## Remaining work

Q203 is only partly implemented. The last-carried-Rock old-identity edge,
non-Rock stack removal/activation, pickup and deferred deletion release,
and full saved-contact reattachment rules still need implementation.
The older non-Rock drop tests describe legacy behavior, not proof of
the newly studied original lifecycle. Other creation paths, including
initial loading and direct combat item allocation, need their own review.
Item collision-type/mask overrides are not tracked; the current live-mask
checks cover trap masks. Exact spatial candidate traversal under a callback
relocating an item is not established by these focused checks.

The Rock aim and trajectory still use the prior implementation; original
trajectory fidelity remains Q202. Passing this puzzle does not prove the
full original projectile simulation or complete Dead City. Portal traversal,
the skull/feather/scales/Sylph quest and later campaign branches remain open.
No manual or visual test was performed. Implementation B used A's prose
notes and installed data, without reading the original executable.
