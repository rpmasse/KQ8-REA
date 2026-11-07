# Quest-item application lifecycle

This M4 batch implements the shared replacement rule described by study
session A in [inventory.md, rules 14–17](behaviour/inventory.md): applying
a quest item first unapplies other applied quest items in Connor's
inventory. It uses the existing unapply path to remove their statistics,
timer and status icon before applying the new item once.

Applying is separate from holding an item in the pointer. The bar still
holds quest items; this change does not add an application action to the
bar or invent a campaign trigger. The study note does not establish the
original callers that apply quest items. The Candle's pickup experience
remains governed by its existing separate rule.

The implementation and grouped check are in
`crates/kq8-core/src/connor_inventory.rs`. No save-format, journey,
inventory-bar or viewer code changes are needed.

## Grouped automated check

Run from the repository root:

```powershell
cargo test --locked --release -p kq8-core connor_inventory::tests -- --nocapture
```

The target runs the existing inventory family regressions and
`quest_application_replacement_lifecycle_matrix`. That matrix combines
permanent and timed old/new quest items into four replacement cases.
Synthetic item types with distinguishable statistics make the shared
rule observable without introducing a real quest or relying on the
Candle's separate pickup reward.

Each case checks:

- Removal of the old statistics, timer and icon, and exactly one new
  application, while retaining unrelated equipment and a running potion.
- Unchanged ownership, quantities, quest-strip order and held item.
- Repeated application changes neither statistics nor timer/icon state.
- JSON round-trip through the existing `Journey` carried-state record,
  restoration on a different update clock, and removal through the
  ordinary `dropItem` command.
- No later subtraction from a removed timer; otherwise the timed
  replacement expires once and the permanent replacement stays applied.

The first family run reused the coordinated core build in 0.23 seconds:
the 16 existing inventory tests passed, and the new matrix failed at its
ordinary-drop assertion because the test command omitted the required
`KQMonster::` prefix. That fixture command was corrected; the production
implementation was unchanged. The failed run is retained at
`kq8-artifacts/inventory-application.log`.

The corrected matrix passed all four replacement cases on 2026-11-07:
one test passed, zero failed, in 0.00 seconds after a 56.55-second core
rebuild. Its command and retained output are:

```powershell
cargo test --locked --release -p kq8-core quest_application_replacement_lifecycle_matrix -- --nocapture
```

`kq8-artifacts/inventory-application-corrected.log` records that result.
The 16 successful existing tests were not repeated. No native game
launch or manual test was performed. The completed bar-input evidence in
[inventory-empty-slot.md](inventory-empty-slot.md) is reused.

## Limits

This is shared-core behavior and carried-state serialization coverage,
not a fresh-process viewer save/load check or an original-engine
execution comparison. It does not establish the untraced campaign
application triggers, armour presentation, or overall M4 completion.
The GOG installation is not accessed or modified by this headless test.
