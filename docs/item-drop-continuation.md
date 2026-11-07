# Carried-item removal and drop continuation

This batch implements the category and ordering rules in
[items.md rule 11](behaviour/items.md) and
[traps.md Q203](behaviour/traps.md#recoverable-rock-registration-and-removal-q203-2026-11-07).
It does not change Rock aiming or flight; those are a separate study.

## Behavior

Common removal reduces a larger carried stack by one. Removing a last
item puts that same quantity-one identity into world ownership at its
stored pose. Any active registration callbacks finish before the command
sets `NewInvItem` to that selected identity.

The Rock category then creates a fresh quantity-one landing item. Its
registration callbacks finish before `NewInvItem` becomes the fresh name.
The old last-Rock identity remains world-owned at its stored pose, as the
original static study establishes. It is not silently deleted or reused.
Non-Rocks retain the selected identity: a larger stack remains carried
with its new stored pose, and a last item moves to the destination without
a second contact probe merely because it was placed there.

The carried pose is serialized with the item through disk and journey or
regional item collections. Carrying resets it to the origin. Old Rust
saves omitted this pose; they use an explicit origin fallback, which
cannot recover an omitted historical transform.

Immediate item registration and the ordinary frame's item distance test
share the effective reference point, visible distance and forced-near
settings. The initial reference comes from the loaded Connor position,
with a camera fallback. Frame updates publish the actual selected Connor
or camera reference. Script requests do not apply a next-frame reference
or forced-near change early. Hidden items stay off. Item distance uses
the original fixed, unturned BOUNDS sphere and 0.83 near threshold.
Stationary items changing from inactive to active register through the
same command-continuation mechanism as new items.

## Verification

The core registration batch passes **10/10**, including the new grouped
drop matrix, after a 60-second release compilation. The two corrected
drop-command tests also pass from the same compiled executable without
another Cargo build. Evidence:

- `kq8-artifacts/logs/item-drop-registration-matrix.log`
- `kq8-artifacts/logs/item-drop-non-rock.log`
- `kq8-artifacts/logs/item-drop-equipment.log`

```powershell
cargo test --locked --release -p kq8-core item_registration_tests:: -- --nocapture
# The same compiled executable, without rebuilding:
& target/release/deps/kq8_core-4036e6506ba00dad.exe scripted_non_rock_drop_preserves_the_selected_identity --nocapture
& target/release/deps/kq8_core-4036e6506ba00dad.exe scripted_drop_unapplies_equipment_only_after_validating_placement --nocapture
```

The shared registration path changed, so this one run includes its
existing affected checks. The new grouped matrix covers Rock/non-Rock,
one/many quantities, active/inactive old pose, nested callback mutation,
exact identity and `NewInvItem` ordering, later stationary activation,
hidden/forced-near behavior, distance boundaries, and serialized poses.
The prior Crystal-splitting expectation is replaced because it contradicted
the original category-specific rule. The equipment case now completes the
real command continuation rather than inspecting an unfinished host call.

No previously successful native Rock route is repeated, and no manual
or visual test is used.

The narrow viewer hooks and snapshot validation compile in the shared
viewer test build, which passes in 1m53s. Log:
`kq8-artifacts/logs/ordered-loading-shared-viewer-build.log`.
Their running-viewer behavior is not exercised by that compilation.

## Remaining boundaries

The current bulk loader initializes existing item activity without
replaying contacts. Complete original ordering of permanent/restored
object registration, saved contact reattachment and object Init remains
the shared Q201 loading work. This batch does not establish full original
save or startup parity. The static original study has not been compared
with a native original-game last-Rock run. No such comparison is claimed.

General item collision-type/mask overrides and exact spatial candidate
traversal when callbacks move an object remain outside this batch.
Existing registration callbacks still read live state one at a time.
