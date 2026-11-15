# King's Quest: Mask of Eternity — Rust recreation

KQ8-REA is an unofficial project to recreate Sierra's 1998 *King's Quest: Mask of Eternity* in Rust. Its first goal is to match the original Windows game from the opening menu through the ending. That faithful version will provide the foundation for a later, modernized edition.

## Where it comes from

This project builds on [Eric Peracchi Pisoni's KQ8R](https://github.com/Eric-lz/KQ8R), a Rust rewrite of the game. KQ8-REA continues that work in this fork. The goal is to use [REA (Reverse Engineering Anything)](https://github.com/morluto/rea) where useful to understand the original engine and turn its behavior into a Rust version. REA is a planned research aid; it has not been integrated or run for this project yet.

## Progress

The current Windows development build loads all 12 regions. Daventry and parts of the campaign have automated checks, but the game is not yet recreated through the ending.

| Phase | Status | Scope |
| --- | ---: | --- |
| M0 · Read game data | 100% | Original archives and images |
| M1 · Render the world | 100% | Terrain, objects and lighting |
| M2 · Move Connor | 100% | Movement, collision and camera |
| M3 · Bring Daventry to life | 100% | Early scripts, objects and puzzles |
| M4 · Interface and dialogue | 100% | Menus, inventory, dialogue and display controls |
| M5 · Combat | ~65% | Weapons, enemies, damage and death |
| M6 · Campaign and saves | ~28% | Quests, world transitions and the full story; paused |
| M7 · Match the original | 0% formally | Final comparison and Windows release |

Percentages refer to each phase's planned work, not an overall project percentage. Some focused comparisons have been made, but the final M7 review has not begun. See the [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05) and [current status](STATUS.md) for details.

## Bring your own game

**This repository contains no original game files or assets.** To build and play the Rust version, supply your own GOG copy of *Mask of Eternity* (tested with version 1.03). The recreation reads its levels, models, sound and artwork from that installation.

On Windows, install Rust and the Visual Studio C++ build tools, then run these commands in PowerShell from the repository folder. Replace the example path with your GOG installation folder:

```powershell
$env:KQ8_GAME_DIR = "C:\path\to\King's Quest 8 GOG"
cargo build --locked --release -p kq8-viewer
.\target\release\kq8-viewer.exe --windowed
```

## After the faithful recreation

The modernized version could add widescreen and ultrawide resolutions, smoother movement and animation at higher frame rates, and ports to other platforms, including macOS. These are future goals; the current milestone is matching the original game.

For developer commands, see [automated checks](docs/automated-checks.md). For sources and acknowledgments, see [credits](CREDITS.md) and [provenance](docs/provenance/README.md).
