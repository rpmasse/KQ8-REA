# Empty quest-strip cells and the held item

The original inventory rules distinguish the two mouse buttons:
[inventory.md rule 10](behaviour/inventory.md#clicking-an-item-in-the-bars-left-button)
says that right-clicking an empty slot does nothing. Rule 12 permits a
left click on the empty place to return a held quest item to the strip.

The viewer previously used the same fallback for both buttons. While a
quest item was held as the pointer, every empty quest cell resolved to
that item, so a right click incorrectly described it. The shared lookup
now receives the button and uses the held-item fallback only for a left
click. Occupied cells still resolve to their displayed item. Both the
physical-pointer path and the unattended named-control path use this
lookup; no inventory contents or item-application rules change.

## Automated check

The regression loads the genuine GoodSkullReload campaign save, with
Piece1 and GoodSkull as its two quest entries. It describes the visible
skull, holds it, right-clicks empty slot seven, left-clicks that same
empty slot to put the skull back, then right-clicks it again and saves.
The expected result is one description from the occupied cell, two
silent empty-cell right clicks, the original held/returned item, an
empty final hand and unchanged inventory, stats and quest flags.
AI and god-mode settings remain those of the earned save; the GOG
Options.cs must remain byte-for-byte unchanged.

```powershell
$env:KQ8_GAME_DIR=(Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
$env:KQ8_INVENTORY_SLOT=(Resolve-Path -LiteralPath 'kq8-artifacts/earned-good-skull/1794047529603851200/slots/slot-3.json').Path
cargo test --locked --release -p kq8-viewer --test inventory_empty_slot -- --ignored --nocapture
```

Before the fix, the native viewer completes, but the regression fails
in 10.62 seconds because it records two descriptions instead of one.
Evidence: inventory-empty-slot-before.log and
inventory-empty-slot/1794048528528063200. This is the reproduced bug,
not a failed gameplay route.

After the fix, the native viewer passes in the 11.02-second run
inventory-empty-slot/1794048691212645600. It records only the occupied
cell's description at update 10, holds the skull at 15, ignores the
empty-cell right clicks at 20 and 30, and returns the skull at 25.
The integration test then fails on its old exact log substring, which
omitted the diagnostic's `before update N` text. Cargo had compiled that
test target before the assertion was corrected during the viewer build.
The native behavior is successful; this second failure is in the check.

The corrected assertions are evaluated against the existing logs and
save, including unchanged items, stats, quest flags, AI/god settings and
an empty final hand. They pass without relaunching the game. The current
test target also compiles successfully with `--no-run` in 1.79 seconds.
The first post-fix build took 1 minute 57 seconds. Evidence:
inventory-empty-slot-after.log (native success and stale assertion),
inventory-empty-slot-recorded-check.log (recorded checks pass), and
inventory-empty-slot-compile.log (corrected target compiles).

The final Options.cs byte-comparison assertion was not reached by the
recorded test, so that assertion is **not tested** here; the original
file's modification timestamp remains August 23, 2010. The native route
is not repeated solely to rerun the corrected assertion. The compiled
entry point above retains the complete check for future uses.

The test uses named panel inputs and numeric/event logs. Physical
pointer hit testing, original-runtime comparison and unrelated inventory
branches are **not tested** by this check. No manual testing is used.
