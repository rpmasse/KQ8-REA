# King's Quest: Mask of Eternity — Rust recreation for Windows

KQ8-REA recreates Sierra's 1998 *King's Quest: Mask of Eternity* in Rust. The engine reads the original levels, models, scripts, audio and interface assets from your own GOG installation. The target is the original Windows game, including its controls, timing, 4:3 presentation, and the full campaign from New Game through the ending.

**Development build.** Daventry is the most complete region. All 12 worlds open in the Windows build. You can save in Daventry, quit, and load the slot from a fresh launch. The complete campaign and saving during encounters are still in development. The fork incorporates the [October 31 upstream revision](https://github.com/Eric-lz/KQ8R/commit/5ea5a46f791a637b2d2e231e56195726050e57cb).

## Project status

| Stage | Current implementation | Remaining work |
| --- | --- | --- |
| M0–M3 · Data, rendering and movement | Readers for the original assets, world rendering, Connor's movement and collision, and early Daventry scripts and puzzles. | Reproduce the inherited checks on the target Windows build and track exceptions by region. |
| M4 · Interface and dialogue | New Game and difficulty, pause/resume, restart, original Save and Load screens with names and thumbnails, HUD, speech, captions and some conversations. | Options, Replay Intro, remaining inventory and dialogue behavior, and full menu fidelity. |
| M5 · Combat | Enemy pursuit, melee and some ranged attacks, throwing and retrieving rocks, damage, potions, scripted equipment drops and a death-menu entry. | Verify weapon controls and trajectories, enemy-specific encounters and death recovery. |
| M6 · Campaign and saves | Disk slots restore Connor, inventory and supported quest and region state. Travel, maps, selected terrain switches and Temple 4's final actor loading work. | Restore active scripts, combat and cutscenes. Complete region quests, optional branches, boss sequences, movies and the ending. |
| M7 · Fidelity | Original behavior notes and focused comparisons are available. | Verify the completed recreation against the original game. |

The [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05) defines each milestone and its automated checks. Manual playtesting begins after the Rust implementation is feature-complete.

## Build and run

Use Windows, the stable Rust toolchain for MSVC, the Visual Studio C++ build tools and Windows SDK, and a GOG installation of the game (version 1.3, also called 1.03). Run these commands in PowerShell from the repository root. Set the path to the game directory that contains `data` and `Mask.exe`.

```powershell
$env:KQ8_GAME_DIR = "C:\Games\King's Quest 8"
cargo build --locked --release -p kq8-viewer
.\target\release\kq8-viewer.exe
```

The game opens at the main menu. Choose **New** and a difficulty to enter Daventry. Escape opens the menu during play. For a development run, add `--world Daventry` to start directly in the region. The viewer also accepts `--game <folder>` instead of the environment variable. Run `cargo play --help` for supported arguments. The [viewer reference](docs/FEATURES.md) covers controls and console commands.

Choose **Save** from the pause menu and name the slot. **Load** is available from the title or pause menu. The recreation stores its own saves in `kq8-artifacts/saves` under the launch directory. Set `KQ8_SAVE_DIR` to use a fixed location across launches. Support for importing original Sierra saves is still open.

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
cargo test --locked -p kq8-viewer --test rock_throw -- --ignored
cargo test --locked -p kq8-viewer --test regions -- --ignored --nocapture
cargo test --locked -p kq8-core original_barren_dock_script -- --ignored
```

A world loading successfully verifies that it can be opened. Campaign completion also requires ordinary controls to advance the quests, save and restore their consequences, defeat the final boss and reach the ending. The [script coverage report](docs/SCRIPT-COVERAGE.md) inventories commands for that work and explains what its probes measure.

## Source and research

| Component | Responsibility |
| --- | --- |
| `kq8-data` | Reads and decodes files from the original game. |
| `kq8-core` | Implements game rules, scripts, movement and combat. |
| `kq8-viewer` | Runs the game window, input, rendering and interface. |
| `kq8-audio` | Mixes and plays audio. |
| `kq8-cli` | Inspects game data and runs verification tools. |

This fork builds on [Eric Peracchi Pisoni's KQ8R](https://github.com/Eric-lz/KQ8R). The [research workflow](docs/SESSIONS.md) explains how behavior observed in the original becomes a written specification for the Rust implementation. See [provenance](docs/provenance/README.md), the [development log](MODLOG.md) and [credits](CREDITS.md) for sources and contributions. This is an unofficial fan project. Players supply their own original game files.
