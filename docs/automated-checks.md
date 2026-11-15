## Automated checks

Use the same game-data path for these commands. The workspace command includes the viewer tests. The archive check reads the local game data. The last command launches Daventry and exits after 120 rendered frames.

```powershell
cargo test --locked --workspace
cargo run --locked --release -p kq8-cli -- vol-verify
.\target\release\kq8-viewer.exe --world Daventry --frames 120
```

The following checks use the installed game assets and run unattended. The menu and region tests require a graphics device. The region test opens each of the 12 regions and renders 30 frames. The Barren test executes the original dock script from the player's archive and checks which gong trigger it selects.

```powershell
cargo test --locked -p kq8-viewer campaign_tests -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test main_menu -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test options_menu -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test graphics_preferences -- --ignored
cargo test --locked -p kq8-viewer --test shadow_checkpoint -- --ignored
cargo test --locked -p kq8-viewer --test speech_checkpoint -- --ignored
cargo test --locked -p kq8-viewer --test conversation_checkpoint -- --ignored
cargo test --locked --release -p kq8-viewer --test route_conversation_input -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test connor_weapon_checkpoint -- --ignored
cargo test --locked -p kq8-data connor_view_ -- --include-ignored
cargo test --locked -p kq8-viewer --test keyboard_menu -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test credits_menu -- --ignored
cargo test --locked -p kq8-viewer --test starting_help -- --ignored
cargo test --locked -p kq8-viewer --test inventory_tutorials -- --ignored
cargo test --locked -p kq8-viewer --test movies -- --ignored --nocapture
cargo test --locked -p kq8-viewer --test rock_throw -- --ignored
cargo test --locked -p kq8-viewer --test rock_flight_checkpoint -- --ignored
cargo test --locked -p kq8-viewer --test weeper -- --ignored
cargo test --locked -p kq8-viewer --test weeper_flight_checkpoint -- --ignored --nocapture
cargo test --locked -p kq8-viewer --test kept_interactions -- --ignored
cargo test --locked -p kq8-viewer --test cottage_journey -- --ignored
cargo test --locked -p kq8-viewer --test daventry_wizard -- --ignored
cargo test --locked -p kq8-viewer --test daventry_windmill -- --ignored
cargo test --locked -p kq8-viewer --test daventry_hero_ring breakable_route_dispatches -- --ignored
cargo test --locked -p kq8-viewer --test daventry_cottage_shield earned_crystal_to_hero_ring_and_fresh_reload -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --test daventry_cottage_shield earned_hero_ring_to_watermill_tree -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --test daventry_cottage_shield earned_mill_rope_on_live_and_reloaded_scenery -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --test daventry_cottage_shield church_rope_is_usable_after_a_fresh_launch_inside_its_trap -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --test daventry_cottage_shield church_climb_resumes_from_its_saved_frame_on_the_wall -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --test daventry_cottage_shield first_person:: -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test connor_death_checkpoint first_person_death_selects_the_third_person_pose_and_resumes_once -- --ignored --exact
cargo test --locked -p kq8-viewer --test daventry_cottage_shield church::earned_roof_to_candle_and_fresh_reload -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --test daventry_cottage_shield church::earned_candle_to_urn_and_open_tomb_door -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --test daventry_cottage_shield shadow::wizard_accepts_the_earned_ring_and_candle -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --test daventry_cottage_shield castle::waterfall_rope_and_brick_puzzle -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --test daventry_cottage_shield castle_interior::fireplace_passage_to_ashes_and_fresh_reload -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --test daventry_cottage_shield wizard_ring::ring_opens_dead_city_and_arrival_survives_reload -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --test daventry_cottage_shield dead_city::restored_letter_sequence_earns_deaths_key -- --ignored --exact --test-threads=1
cargo test --locked -p kq8-viewer --bin kq8-viewer fallen_tree_and_dock_use_the_same_surfaces_on_script_load_and_revisit -- --ignored --nocapture
cargo test --locked -p kq8-viewer --test raven_flight -- --ignored
cargo test --locked -p kq8-viewer --test spawned_monsters -- --ignored
cargo test --locked -p kq8-viewer --test placed_characters -- --ignored
cargo test --locked -p kq8-viewer --test death_checkpoint -- --ignored
cargo test --locked -p kq8-viewer --test connor_death_checkpoint -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test choice_checkpoint -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test henchman_combat -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test ordinary_once_checkpoint -- --ignored
cargo test --locked -p kq8-viewer --test alternate_interior -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test connor_swing_checkpoint -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test puzzle_timer -- --ignored
cargo test --locked -p kq8-viewer --test regions -- --ignored --test-threads=1 --nocapture
cargo test --locked -p kq8-core original_barren_dock_script -- --ignored
cargo test --locked -p kq8-core original_gnome_rock_and_dead_city -- --ignored
```

The [Daventry campaign route](daventry-progression.md) starts at New Game. An unattended player follows the road around the farmhouse using the bound movement keys, talks to the wizard and saves after his last line. The save contains the mask, the first piece and the Magic Map awarded by his original scripts. A second process follows the normal route to the windmill, fights the archer, pushes the bale, takes the Small Axe and saves. A third process restores Connor standing on the moved bale, with the axe in its scripted position; a second save preserves both placements.

The separate cottage test collects five mushrooms, consumes one, saves and quits. A fresh process restores four mushrooms and the open cottage door, opens the pub chest, travels through both castle portal prompts, and saves back in Daventry. This test disables enemy AI, moves Connor between landmarks and applies damage so he can use a healing item.

Two Dead City checks start with the original portal's arrival cue. One gives Connor the feather, uses it on the right dish and steers him onto the left. The original scripts then lower five river sections. The other boards the right dish, steps off and walks out of the puzzle. Getting the feather during campaign play and saving mid-puzzle still need coverage.

The Temple 4 scenarios check the altar’s changes to Lucreto’s movements and the animated sequences for victory and defeat. All three `KQMonster::setState Connor Die Now` branches hold Connor and the final camera shot for three seconds, then open the paused death menu. The unattended Windows route checked the empty-hand ending; the sword and bow endings passed original-data simulation tests. Separate movie checks cover title replay, the middle film returning to play, and the ending film followed by scored credits. Reaching that ending in an uninterrupted campaign remains open. The [script coverage report](SCRIPT-COVERAGE.md) lists commands across all worlds.

For detailed behavior and evidence, see [Daventry progression](daventry-progression.md), [feature notes](FEATURES.md), and the [development log](../MODLOG.md).
