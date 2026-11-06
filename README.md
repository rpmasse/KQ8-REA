# King's Quest: Mask of Eternity — Rust recreation for Windows

KQ8-REA recreates Sierra's 1998 *King's Quest: Mask of Eternity* in Rust. The engine reads the original levels, models, scripts, audio and interface assets from your own GOG installation. The target is the original Windows game, including its controls, timing, 4:3 presentation, and the full campaign from New Game through the ending.

**Development build.** Daventry is the most complete region. All 12 worlds open in the Windows build. You can save in Daventry, quit, and load the slot from a fresh launch. The complete campaign and saving during encounters are still in development. The fork incorporates the [October 31 upstream revision](https://github.com/Eric-lz/KQ8R/commit/5ea5a46f791a637b2d2e231e56195726050e57cb).

## Project status

| Stage | Current implementation | Remaining work |
| --- | --- | --- |
| M0–M3 · Data, rendering and movement | Readers for the original assets, world rendering, Connor's movement and collision, and early Daventry scripts and puzzles. | Reproduce the inherited checks on the target Windows build and track exceptions by region. |
| M4 · Interface and dialogue | Title, pause, Save, Load, Options, Graphics, Keyboard, scripted Help and F1 work. Replay Intro plays the original film. Credits displays the original 25 cards and team photograph. The HUD, speech, captions and some conversations work. Graphics saves resolution, lighting, shadow and brightness choices; lighting applies immediately, shadow categories apply when play resumes, and fresh loads apply resolution and the palette component of brightness. Scripts can suppress shadows independently, and saves retain that switch. | Graphics mode selection, display gamma, live brightness and resolution, exact colour rounding, inventory and dialogue behavior, and exact credits timing and score calculation. |
| M5 · Combat | Dead City's Weeper and Tower Skeleton have encounter checks; all six script-created monster types spawn and draw, and nearby copies now update as characters with their face materials. In Temple 4 Lucreto switches between melee and ranged attacks using the original distance rule. A melee stagger can push him into the vortex. Scripted stat changes affect live combat stats and send monsters and Connor through their death sequences. All three scripted Connor defeat branches open the death menu after the original three-second hold. | Complete weapon controls and trajectories, remaining enemies, encounter timing and ordinary death recovery. |
| M6 · Campaign and saves | Automated routes cover the wizard's rewards, the windmill Small Axe, the Daventry castle round trip and the Dead City scales. Saves restore moved windmill props, timed puzzle events, a scripted raven in flight, original and script-created monsters, unfinished deaths, Connor's live melee swing and standing crossbow attack, queued melee and ranged combat cues, live bolts, moving-shot timing, Small Axe and Small Crossbow weapon swaps, and the Weeper drain with Connor's reaction phase. A scripted hit also restores Connor's reaction, hit timing and last hit kind. Active speech resumes its voice offset, sound origin, caption and remaining queue. Wizard conversations retain approach, weapon-away/out phases, goodbye/pause/return queues, camera mode and HUD state across reloads, then return control. Scripted camera flights retain their remaining steps and destination list for later cuts. Scripted footsteps, occupied trigger state and flagged trigger callbacks also survive disk reloads. Persistent ordinary objects retain damage and hit history. New ordinary copies use their original file stats, and reused names stay live through saves. The final movie can lead into scored credits. | Check other partners and conversation branches, actual camera drags, remaining trigger/cutscene state, and other weapon swaps and combat actions. Complete region quests, optional branches and boss sequences in an uninterrupted campaign. |
| M7 · Fidelity | Original behavior notes and focused comparisons are available. | Verify the completed recreation against the original game. |

The [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05) defines each milestone and its automated checks. Manual playtesting begins after the Rust implementation is feature-complete.

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
cargo test --locked -p kq8-viewer --test connor_weapon_checkpoint -- --ignored
cargo test --locked -p kq8-viewer --test keyboard_menu -- --ignored --test-threads=1
cargo test --locked -p kq8-viewer --test credits_menu -- --ignored
cargo test --locked -p kq8-viewer --test starting_help -- --ignored
cargo test --locked -p kq8-viewer --test movies -- --ignored --nocapture
cargo test --locked -p kq8-viewer --test rock_throw -- --ignored
cargo test --locked -p kq8-viewer --test weeper -- --ignored
cargo test --locked -p kq8-viewer --test kept_interactions -- --ignored
cargo test --locked -p kq8-viewer --test cottage_journey -- --ignored
cargo test --locked -p kq8-viewer --test daventry_wizard -- --ignored
cargo test --locked -p kq8-viewer --test daventry_windmill -- --ignored
cargo test --locked -p kq8-viewer --test raven_flight -- --ignored
cargo test --locked -p kq8-viewer --test spawned_monsters -- --ignored
cargo test --locked -p kq8-viewer --test placed_characters -- --ignored
cargo test --locked -p kq8-viewer --test death_checkpoint -- --ignored
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
