# King's Quest: Mask of Eternity in Rust

**KQ8-REA** is an unofficial recreation of Sierra's 1998 game *King's Quest: Mask of Eternity*. This fork builds on [Eric Peracchi Pisoni's KQ8R](https://github.com/Eric-lz/KQ8R), an earlier Rust rewrite. The first goal is to recreate the complete original Windows game.

The goal is to use [REA (Reverse Engineering Anything)](https://github.com/morluto/rea) to study the original game and guide a Rust conversion that reads the player's game data. REA has not yet been integrated or run. That Rust version will be the foundation for a modernized edition.

## Project phases

| Phase | Progress |
| --- | ---: |
| M0 · Read original game data | 100% |
| M1 · Render the world | 100% |
| M2 · Movement and camera | 100% |
| M3 · Daventry gameplay | 100% |
| M4 · Menus, inventory and dialogue | 100% |
| M5 · Combat | ~65% |
| M6 · Full campaign and saves | ~28% |
| M7 · Final comparison and Windows release | 0% |

These are estimates for each phase, not an overall completion percentage. The Windows build loads all 12 regions, but the full game cannot yet be played through to the ending. See [current status](STATUS.md) and the [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05) for details.

## Bring your own game

**This repository contains no original game files or assets.** You need your own GOG copy of *Mask of Eternity* to build and run the Rust version. The project has been tested with GOG version 1.03.

On Windows, install Rust and the Visual Studio C++ build tools. Then run this in PowerShell from the repository folder, replacing the example path with your GOG installation:

```powershell
$env:KQ8_GAME_DIR = "C:\path\to\King's Quest 8 GOG"
cargo build --locked --release -p kq8-viewer
.\target\release\kq8-viewer.exe --windowed
```

## After the recreation

Future enhancements could include widescreen and ultrawide resolutions, higher frame rates for movement and animation, and ports to other platforms, including macOS.

See [credits](CREDITS.md) for the project's sources and [automated checks](docs/automated-checks.md) for developer commands.
