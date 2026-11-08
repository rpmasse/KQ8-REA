# Inventory display and held-item selection

The inventory bar now selects its represented armor and weapon from carried
objects in their original addition order. It uses the studied body/under-armor
rules, rather than ranking armor by protection or inferring a full suit from
accessories. Gold and Rock cells show a literal zero; empty health and potion
counts stay blank. Gold pictures change at 1/25/50 and Rocks at 1/5/15.

Picture changes share the original 250 ms clock bands. A control finishes
after four observed returns from the empty picture; unseen bands are not
counted as completed flashes. Same-object count changes do not restart this
sequence. Held money retains the normal picture threshold and full count.

Ordinary use-at-once selections return a previously held item before applying
their own category, including an already-active potion's no-op. Holdable swaps,
same-item returns and empty known-cell clicks share the hand-clearing path.
Right-click descriptions, cursorless items, rope restrictions and Mask's
special menu action retain their separate rules.

## Automated evidence, 2026-11-08

One shared release test build passed in **2m37s**. Three inventory core checks
and two graphics-focus checks passed **5/5 in 0.40s**, first attempt. The HUD
check loads original GUI tags, pictures and fonts, then renders through the
production CPU canvas and dispatches named inventory actions. It passed
**1/1 in 0.05s**, first attempt, without a window or visual inspection.

```powershell
cargo test --locked --release -p kq8-core ordinary_bar_selection_held_category_and_empty_matrix
cargo test --locked --release -p kq8-core inventory_hud_order_count_and_observed_clock_matrix
cargo test --locked --release -p kq8-core the_slots_show_what_he_wears
cargo test --locked --release -p kq8-viewer --bin kq8-viewer original_inventory_hud_binding_count_blink_render_and_dispatch_matrix -- --ignored --nocapture
```

Set `KQ8_GAME_DIR` to the absolute, read-only GOG installation. Each command
should report a passing test. Retained logs are
`kq8-artifacts/logs/m4-hud-focus-build.log`, `m4-hud-focus-core.log` and
`m4-hud-original-data.log`. Root ran the compiled test binaries directly
after the shared build; these Cargo commands reproduce the same filters.
Previously successful pickup and inventory-application routes were not repeated.

Source: Study A's `behaviour/interface.md` rules 35, 39 and 43–45 and Q109/Q110
follow-up, plus `behaviour/inventory.md` rule 9. No executable was read by B.
New native dispatch, original-runtime comparison, unusual simultaneous hidden
panel updates and held-item lifetime through cutscenes/death/world changes are
**not tested** by this batch. No save format or texture resolution was changed.
