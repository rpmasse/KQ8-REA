# King's Quest: Mask of Eternity — Rust recreation for Windows

KQ8-REA recreates Sierra's 1998 *King's Quest: Mask of Eternity* in Rust. The engine reads the original levels, models, scripts, audio and interface assets from your own GOG installation. The target is the original Windows game, including its controls, timing, 4:3 presentation, and the full campaign from New Game through the ending.

**Development build.** All 12 regions load and render on Windows. An unattended test started a fresh Normal New Game, earned the dead hero’s ring, the watermill Rope and Hook and the church Candle, then reloaded all three. It passed in 24 minutes 54 seconds across 18 Windows launches using only saves created during the test. The rest of the campaign through the ending remains open. This fork incorporates the [October 31 upstream revision](https://github.com/Eric-lz/KQ8R/commit/5ea5a46f791a637b2d2e231e56195726050e57cb).

## Project status

| Stage | What works | What remains |
| --- | --- | --- |
| M0–M3 · Data, worlds and movement | The engine reads the original assets, opens every region and runs Connor’s movement, collision and early Daventry scripts. | Recheck inherited behavior in the Windows build and record differences by region. |
| M4 · Interface and dialogue | Title, pause, named saves, Options, Keyboard, HUD, captions, inventory tutorials and the Wizard conversation work. Menu masks and HUD flashes use the original ten-frame-per-second timing. Credits shows the 25 original cards and team photograph. Lighting, shadows and indexed palette brightness update in play. | Finish graphics mode and live resolution behavior, software palettes, remaining inventory and dialogue branches, credits timing and score calculation; resolve F2 during movement transitions. |
| M5 · Combat | First-person axe strikes and Small Crossbow shots use Connor’s original animations. Equipped attacks, pending deaths and the three-second game-over hold have save and reload checks. The Weeper, Tower Skeleton, Lucreto and six script-created monster types have focused checks. | Complete the remaining weapons, enemies, bosses and controller transitions. Verify full encounters during campaign play. |
| M6 · Campaign and saves | Using earned campaign saves, Connor completes the Wizard’s Ring of Light quest, travels through Castle Daventry and enters Dead City. Focused checks cover the scales, both Temple 4 outcomes and saves during encounters and cutscenes. | Complete Dead City and the remaining regional quests, optional branches and bosses. Verify one continuous New Game through the ending and credits. |
| M7 · Original-game fidelity | The project has original behavior notes and focused automated comparisons. | Compare the feature-complete Rust recreation against the GOG release. |

**Current campaign boundary.** In Dead City, Connor solves the paddle and letter puzzles, unlocks the spike gate and clears the gate, northern junction and outer mace guard encounters. A saved western-patrol victory continues through the river Weeper, boathouse guard, four Ferryman topics, western Rock pickup, all four Sylph pressure plates and her three introductory topics with earned equipment, original rewards and verified reloads; replaying the earlier patrol fight remains unreliable. The Sylph item quest, later Ferryman branches, western maze, skull, feather and scales still need a campaign route. [Dead City route](docs/dead-city-progression.md)

**How the original is used.** The Rust engine reads scripts, models, audio, interface files and other assets from the player’s GOG installation. A separate study session investigates compiled engine rules and records them in prose. The implementation uses those notes and the installed data. [Study and implementation workflow](docs/SESSIONS.md)

The loader now gives the installed regional patch files priority over archive copies, with an automated check of all 22 overrides and the patched portal callback. [File selection](docs/world-file-selection.md)

Press **F2** during play to switch between first and third person. The
viewer selects Connor’s arm and weapon animations from the tables in
your installed `GAME/Kq/Connor.kq`. Drawing a weapon, using a rope and
loading a save can all cross a view change. Near-wall clipping and more
weapons still need comparison. [First-person controls and evidence](docs/FEATURES.md#connors-view-specific-animation-files-2026-11-07)

The [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05) defines each milestone and its automated checks. The [feature reference](docs/FEATURES.md) gives subsystem details and the [status log](STATUS.md) records verified results and open cases. Manual playtesting begins after the Rust implementation is feature-complete.

## Build and run

Use Windows, the stable Rust toolchain for MSVC, the Visual Studio C++ build tools and Windows SDK, and a GOG installation of the game (version 1.3, also called 1.03). Run these commands in PowerShell from the repository root. The example uses the development layout, with `KQ8-REA` and `King's Quest 8 GOG` side by side. For another layout, set `KQ8_GAME_DIR` to the folder containing `data` and `Mask.exe`.

```powershell
$env:KQ8_GAME_DIR = (Resolve-Path "..\King's Quest 8 GOG").Path
cargo build --locked --release -p kq8-viewer
.\target\release\kq8-viewer.exe
```

The game opens at the main menu. Choose **New** and a difficulty to enter Daventry. Escape opens the menu during play. For a development run, add `--world Daventry` to start directly in the region. The viewer also accepts `--game <folder>` instead of the environment variable. Run `cargo play --help` for supported arguments. The [viewer reference](docs/FEATURES.md) covers controls and console commands.

Choose **Save** from the pause menu and name the slot. **Load** is available from the title or pause menu. The recreation stores its own saves in `kq8-artifacts/saves` under the launch directory. Set `KQ8_SAVE_DIR` to use a fixed location across launches. Support for importing original Sierra saves is still open.

The three original movies use Indeo 5 video. The development build uses FFmpeg to decode the pictures and reads the original stereo soundtrack in Rust. Set `KQ8_FFMPEG` to the full path to a local `ffmpeg.exe`, place it beside the viewer executable, or make it available on `PATH`. Windows acceptance used FFmpeg 7.1. A missing decoder opens a message you can dismiss. Packaging a decoder for the finished Windows release remains open.

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
cargo test --locked -p kq8-viewer --test weeper -- --ignored
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

The [Daventry campaign route](docs/daventry-progression.md) starts at New Game. An unattended player follows the road around the farmhouse using the bound movement keys, talks to the wizard and saves after his last line. The save contains the mask, the first piece and the Magic Map awarded by his original scripts. A second process follows the normal route to the windmill, fights the archer, pushes the bale, takes the Small Axe and saves. A third process restores Connor standing on the moved bale, with the axe in its scripted position; a second save preserves both placements.

The separate cottage test collects five mushrooms, consumes one, saves and quits. A fresh process restores four mushrooms and the open cottage door, opens the pub chest, travels through both castle portal prompts, and saves back in Daventry. This test disables enemy AI, moves Connor between landmarks and applies damage so he can use a healing item.

Two Dead City checks start with the original portal's arrival cue. One gives Connor the feather, uses it on the right dish and steers him onto the left. The original scripts then lower five river sections. The other boards the right dish, steps off and walks out of the puzzle. Getting the feather during campaign play and saving mid-puzzle still need coverage.

The Temple 4 scenarios check the altar’s changes to Lucreto’s movements and the animated sequences for victory and defeat. All three `KQMonster::setState Connor Die Now` branches hold Connor and the final camera shot for three seconds, then open the paused death menu. The unattended Windows route checked the empty-hand ending; the sword and bow endings passed original-data simulation tests. Separate movie checks cover title replay, the middle film returning to play, and the ending film followed by scored credits. Reaching that ending in an uninterrupted campaign remains open. The [script coverage report](docs/SCRIPT-COVERAGE.md) lists commands across all worlds.

## Source and research

Live palette handling is in [live_palette.rs](crates/kq8-viewer/src/live_palette.rs); the [feature notes](docs/FEATURES.md) distinguish tested rules from remaining output differences.

The GOG installation supplies two kinds of evidence. Files such as `Options.cs` and the scripts inside each world’s `.vol` archive contain readable game logic. For example, this command prints Dead City’s scales script directly from the installed archive. Its first numbered cue lowers five sections of the river.

```powershell
cargo run --locked -p kq8-cli -- text vol:DeadCity/kq/SCALE.CS
```

The original engine ships as `Mask.exe`. The [Lucreto study](docs/behaviour/lucreto-forms.md) shows how executable research and game data work together. His character file specifies a switching distance of 2,000. The executable supplies the three-dimensional comparison, its behavior at exactly 2,000 and how one form takes the other's place. The [scripted death study](docs/behaviour/connor-vortex-death.md) records the original three-second hold and death-menu transition. The [research workflow](docs/SESSIONS.md) keeps that investigation separate from writing the Rust code.

| Component | Responsibility |
| --- | --- |
| `kq8-data` | Reads and decodes files from the original game. |
| `kq8-core` | Implements game rules, scripts, movement and combat. |
| `kq8-viewer` | Runs the game window, input, rendering and interface. |
| `kq8-audio` | Mixes and plays audio. |
| `kq8-cli` | Inspects game data and runs verification tools. |

This fork builds on [Eric Peracchi Pisoni's KQ8R](https://github.com/Eric-lz/KQ8R). See [provenance](docs/provenance/README.md), the [development log](MODLOG.md) and [credits](CREDITS.md) for sources and contributions. This is an unofficial fan project. Players supply their own original game files.
