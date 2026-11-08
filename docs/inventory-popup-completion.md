# Inventory selection and popup geometry

Study A's `behaviour/inventory.md` rules 8–9 now identify the Scroll
clear-before-speech behavior, exact refusal tuples, the three restricted Rock
controllers and the rope's independent view lock. The bar frontend supplies
actual controller ownership: Tile, Tile3 and Scales refuse Rock selection;
Tile2 permits it. Generic clearing and swaps do not run the held rope's
put-back action. Only clicking the held rope itself releases that view lock.

Scripted equipment creation establishes explicit placement before exchanging
equipment. Omitted placement is zero, so the displaced old equipment goes to
the origin. The ordinary carry-add reset for the new item remains unchanged.

`world-change.md` rule 26 and `in-game-help.md` supply dynamic popup geometry.
Portal, tutorial, F1 and map-help buttons use the current converted parent
width with fixed pixel widths and heights. Text and button vertical overrides
are final display coordinates; they are not scaled a second time. Drawing,
hit tests and keyboard pointer placement share these rectangles.

`interface.md` rule 12 supplies the top-bar boundary: client row 0 opens,
row 60 does neither, and rows greater than 60 close at both original resolutions.
The implementation reads the Window's physical client sample before GUI
normalization. Bevy documents that getter and its in-client-only scope in
[Window::physical_cursor_position](https://docs.rs/bevy_window/0.20.0/bevy_window/struct.Window.html#method.physical_cursor_position).
The grouped check supplies 640/800 Windows at 100%/150% scale. Original
outside-client capture and native DPI behavior remain unverified.

## Evidence

The core build passes in 1m12s. Scripted equipment placement and its affected
existing equipment check both pass. The selection matrix first failed because
its fixture advanced the diagnostic update count rather than simulation time;
the recent-Scroll speech gate correctly suppressed the line. The fixture now
uses the simulation clock at just below three seconds and exactly three
seconds. Its focused retry passes **1/1**. The viewer inventory dispatch,
popup geometry and client threshold group passes **3/3 in 0.12 seconds**;
popup geometry includes 48 CPU renders across both original sizes.

The first shared build rejected an unnecessary fixture write to a private
rope-history field. Selecting Rope at Top already initializes that field;
removing only the redundant fixture write resolves the error. The corrected
shared build passes in **1m21s**. Production source was unchanged by either
fixture correction. Retain `m4-inventory-popup-build.log` and
`m4-inventory-popup-build2.log`, plus the passing
`m4-selection-core-retry.log` and `m4-inventory-popup-viewer.log`, all under
`kq8-artifacts/logs/`.

Retained logs: `kq8-artifacts/logs/m4-selection-core-build.log` and
`m4-selection-and-phase-core.log`. This run also verified the corrected Talk
completion fixture; see `conversation-admission.md`. Do not repeat the two
successful equipment checks for the unrelated clock-fixture correction.

## Commands

Set `KQ8_GAME_DIR` to the absolute read-only GOG path, and `KQ8_SAVE_DIR` to
an isolated ignored scratch directory for viewer checks. The focused filters
are:

```powershell
cargo test --locked --release -p kq8-core inventory_selection_scroll_rock_owner_and_rope_lock_matrix -- --nocapture
cargo test --locked --release -p kq8-core inventory_scripted_equipment_replacement_placement_matrix -- --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer original_inventory_selection_owner_scroll_and_rope_dispatch_matrix -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer original_popup_geometry_draw_hit_and_keyboard_matrix -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer top_bar_original_client_pixel_threshold_matrix -- --nocapture
```

Each command should report passing tests. The viewer inventory
matrix uses actual controller entry and the ordinary GUI dispatch with empty
render assets; the popup matrix loads original artwork and exercises CPU
drawing and hits. Neither is a native rendering or full campaign claim.
Original resizing while a popup is already open remains unverified.
The newly studied weapon-assignment selection clear is a separate pending
step, so this batch does not close every held-item lifetime rule.
