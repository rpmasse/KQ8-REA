# Temple Armour animation mappings

Implementation plan, 2026-11-08. This is inventory-side M4 work, based on
`docs/behaviour/inventory.md` rule 8, "Temple armour animation changes".
The behavior study is static; original live equip/removal/save scenarios
were not tested.

1. Replace the Temple Armour pickup stub with one shared operation for
   pickup and `KQConner::helmetize`. Change the third-person unarmed list
   to `conner2.anm` and the currently carried short/ranged weapon entries
   to the specified files. Leave other entries and every first-person
   entry intact. Record a list-refresh request independently of weapon
   mode assignment, so the held inventory item is not cleared.
2. Preserve the actual changed entries across Rust save/load and world
   travel. Do not infer them from currently worn armour, undo them on
   removal, or replay pickup effects. Old slots without the new fields
   retain their previous default-map behavior.
3. Restore mappings before loading Connor's models. Load the selected
   third-person base and weapon banks, including an old bank retained by
   the animation controller. Live pickup must make its required banks
   available before refreshing and advancing Connor in that fixed
   update; it must not turn into a draw/put-away sequence.
4. Refresh the active list for the current view/mode and re-request the
   current action through the existing animation controller. Retain its
   ordinary cut-short/transition behavior. Save an outstanding refresh
   and retained old-bank state if a checkpoint falls before completion.
5. Add grouped automated checks for actual equipped-type substitution,
   repeated changes, unchanged first-person/other mappings, held-item
   preservation, removal, Journey serialization, actual original-data
   bank loading, and frontend refresh/checkpoint continuity.

Ownership: the map worker owns `fight.rs`, `connor_inventory.rs`,
`journey.rs`, the serde derive on `ConnorAnimLists`, `load.rs`, and new
viewer child modules/tests. The controller worker owns shared dispatch,
fixed-update integration, save-slot hooks, and existing controller files.
The root coordinates compilation, validation, and completion metadata.

All five implementation steps are complete. The grouped core persistence
matrix and all three viewer refresh/load matrices pass. The core fixture
first supplied an instance instead of a type to `dropItem`; that test-only
correction passed on its focused retry. See
[build, checks and reproduction commands](weapon-admission-completion.md).
No native or manual gameplay test was run.

Missing equipped items/mapping entries and empty filenames are not
established as safe original behavior. The implementation rejects them
explicitly as defensive handling, not a claim of original-game fidelity.
Missing model files report failure and retain the pending refresh.

## Post-validation corrections

The read-only review found two cases not covered by the first matrices.
Repeated `helmetize` operations can give different equipped types the same
third-person filename, while their first-person entries still differ.
The follow-up study in `inventory.md` rule 8 and `ranged-weapons.md` rule
8a establishes that selection uses current mode/view/currently equipped
item, including while an older movement continues. Remove the proposed
drawn-item persistence field; resolve each mode directly from inventory
for both perspective paths. Preserve the pending target mode separately
and retain any old qualified pose through the ordinary movement request.
An explicit refresh clears the animation-owned lock, preserving scripted
handsOff. Exercise the existing F2 key gate rather than bypassing it in
the alias scenario. Before pending `out` can start, resolve its bank from
current equipment; assignment still occurs only at the existing boundary.
The existing Special playback loop does not identify the Swap owner.
Add one runtime owner flag derived from the already saved `readying`
state. Swap-owned Special may advance at movement completion while locked,
and each different requested swap movement sets that lock. Other Special
owners keep their current progression rules. Clear the owner at completion
or an admitted owner change; no extra checkpoint field is needed.
Its retained request name is reconstructed from the saved Special position;
the first Swap request locks independently of any unrelated visible clip.

The legacy late-load catalogue and live inventory mapping scan can request
the same bank in one update. Plan one load per case-insensitive filename
and view, omitting banks already loaded, before allocating models or
materials. Extend the existing refresh/checkpoint and original-data live
loading matrices with repeated aliases, replacement during a pending swap,
and overlapping/stale catalogue requests. These corrections now pass in the
shared 2m41s release build and seven-check viewer group (1.31s). See
`controller-exit-completion.md`. The proposed old drawn-item identity was
discarded before testing after the original study established equipped-item
selection; no such field is persisted by the final implementation.
